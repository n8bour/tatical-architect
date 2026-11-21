// Tactical Bible Bridge — minimal BepInEx 6 (IL2CPP) plugin for FM27, driven by FM27 Tactical Bible (127.0.0.1:27029).
// Trimmed from FM27Bridge (fm27-assistant, MIT, © 2026 arress03). It knows nothing about tactics: it only
//  - learns the references the FM UI uses (GameInteropSubsystem.OpenChannel / SendEvent),
//  - reads data channels like the UI does (READ),
//  - replays orders to the engine like the menus do (SENDJ / WRITE),
//  - records the orders FM sends (EVENTS) so they can be re-captured after a game patch.
// All FM-specific ids (events, properties, keys) live in the app: src/ui/fmbridge.js.
//
// Protocol: one line = one command, one line = the reply ("OK …" / "ERR …" / "PONG …").
//   PING                                 -> PONG ready | PONG waiting
//   VERSION                              -> OK <plugin version> <FM version>
//   REFS                                 -> OK session=<n> <type>=<k=v;…> | …
//   READ type|pairs|prop ;; …            -> OK <json>\u001e<json>…
//   SENDJ <ev> <type> <pairs|-> <json>   -> OK
//   WRITE type|pairs|prop <json>         -> OK
//   EVENTS                               -> OK <json>\u001e<json>…   (orders recorded since the last call)
//   RECORD 0|1                           -> OK
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using BepInEx;
using BepInEx.Unity.IL2CPP;
using FM.GamePlugin;
using HarmonyLib;
using Il2CppInterop.Runtime;
using SI.Core;
using SI.Interop;
using UnityEngine;

namespace TacticalBibleBridge
{
    [BepInPlugin(Guid, "Tactical Bible Bridge", Version)]
    public class Plugin : BasePlugin
    {
        public const string Guid = "fm27.tacticalbible.bridge";
        public const string Version = "0.1.0";
        public const int Port = 27029;
        const uint KeySession = 1198877028;

        internal static GameInteropSubsystem Interop;
        internal static readonly ConcurrentQueue<Action> MainThread = new();
        internal static readonly List<string> Events = new();
        internal static bool Recording = true;
        internal static bool Replaying;
        internal static readonly Dictionary<string, InteropReference> RefByType = new();   // last object seen per reference type
        internal static readonly Dictionary<string, string> PairsByType = new();          // its ids (k=v;k=v)
        internal static readonly Dictionary<string, InteropReference> SeenRefs = new();   // "type|pairs" -> real game object
        internal static readonly Dictionary<long, TypedValue> NumCache = new();           // game numbers already seen (copies)
        internal static int SessionId;

        static string Dir => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "FM27TacticalBible");
        static string RefsPath => Path.Combine(Dir, "bridge_refs.txt");
        static string LogPath => Path.Combine(Dir, "bridge.log");

        public override void Load()
        {
            try { Directory.CreateDirectory(Dir); } catch { }
            try { if (File.Exists(RefsPath)) foreach (var l in File.ReadAllLines(RefsPath)) { var i = l.IndexOf('\t'); if (i > 0) PairsByType[l[..i]] = l[(i + 1)..]; } } catch { }
            new Harmony(Guid).PatchAll(typeof(Patches));
            try { new Harmony(Guid + ".data").PatchAll(typeof(DataPatches)); }
            catch (Exception e) { Write("data reading unavailable: " + e.Message); }
            AddComponent<BridgeBehaviour>();
            new Thread(Server) { IsBackground = true, Name = "TacticalBibleBridge" }.Start();
            Write($"Tactical Bible Bridge {Version} loaded");
        }

        internal static void Write(string s)
        {
            try { File.AppendAllText(LogPath, DateTime.Now.ToString("HH:mm:ss ") + s + Environment.NewLine); } catch { }
        }

        internal static string PairsOf(InteropReference r)
        {
            var ids = r?.m_id; if (ids == null) return "";
            var sb = new StringBuilder();
            for (int i = 0; i < ids.Count; i++) sb.Append(i > 0 ? ";" : "").Append(ids[i].ID).Append('=').Append(ids[i].Value);
            return sb.ToString();
        }

        // ------------------------------------------------------------ learning references
        internal static void Learn(InteropReference r)
        {
            if (r == null) return;
            try
            {
                string type = r.GetIl2CppType().FullName;
                RefByType[type] = r;
                string pairs = PairsOf(r);
                lock (SeenRefs) { if (SeenRefs.Count > 30000) SeenRefs.Clear(); SeenRefs[type + "|" + pairs] = r; }
                var ids = r.m_id;
                if (ids != null) for (int i = 0; i < ids.Count; i++) if (ids[i].ID == KeySession) SessionId = ids[i].Value;
                bool isNew = !PairsByType.ContainsKey(type);
                PairsByType[type] = pairs;
                if (isNew) File.WriteAllLines(RefsPath, PairsByType.Select(kv => kv.Key + "\t" + kv.Value));
            }
            catch { }
        }

        internal static void LearnNumbers(TypedValue v)
        {
            try
            {
                if (v == null || v.IsNull) return;
                var dict = (v.Get() as Il2CppSystem.Object)?.TryCast<Il2CppSystem.Collections.Generic.Dictionary<uint, TypedValue>>();
                if (dict == null) return;
                var e = dict.GetEnumerator();
                while (e.MoveNext())
                {
                    var tv = e.Current.Value;
                    if (tv == null || tv.IsNull || tv.DataType?.FullName != "SI.Core.DynamicNumber") continue;
                    if (long.TryParse(tv.AsString(), out long n) && !NumCache.ContainsKey(n)) NumCache[n] = TypedValue.CreateCopyFrom(tv);
                }
            }
            catch { }
        }

        internal static InteropReference BuildRef(string type, string overridePairs = null)
        {
            if (!PairsByType.TryGetValue(type, out var pairs) && overridePairs == null) throw new Exception("unknown reference: " + type);
            pairs = overridePairs ?? pairs;
            var list = new Il2CppSystem.Collections.Generic.List<InteropReference.Pair>();
            var key = new StringBuilder(type).Append('|');
            foreach (var kv in pairs.Split(';', StringSplitOptions.RemoveEmptyEntries))
            {
                var p = kv.Split('=');
                uint k = uint.Parse(p[0]); int val = int.Parse(p[1]);
                if (k == KeySession && SessionId != 0) val = SessionId;
                if (list.Count > 0) key.Append(';');
                key.Append(k).Append('=').Append(val);
                list.Add(new InteropReference.Pair(k, val));
            }
            lock (SeenRefs) if (SeenRefs.TryGetValue(key.ToString(), out var real)) return real;
            InteropReference r;
            if (RefByType.TryGetValue(type, out var model)) r = model.MemberwiseClone().Cast<InteropReference>();
            else
            {
                Il2CppSystem.Type t = null;
                try { var mt = System.Type.GetType(type + ", FM.UI"); if (mt != null) t = Il2CppType.From(mt); } catch { }
                t ??= Il2CppSystem.Type.GetType(type + ", FM.UI") ?? throw new Exception("type not found: " + type);
                r = Il2CppSystem.Activator.CreateInstance(t).Cast<InteropReference>();
            }
            // id hash computed by the game's own constructor, copied into the object of the right type
            var tmp = new InteropReference(list);
            r.m_id = tmp.m_id;
            r.m_idHash = tmp.m_idHash;
            return r;
        }

        // ------------------------------------------------------------ values
        internal static TypedValue Number(int n)
        {
            try { var dn = new DynamicNumber(); dn.Set(n); return TypedValue.CreateValueType<DynamicNumber>(dn); }
            catch (Exception e) { Write("number (method 1): " + e.Message); }
            try
            {
                var any = NumCache.Values.FirstOrDefault();
                if (any != null)
                {
                    var tv = TypedValue.CreateCopyFrom(any);
                    var dn = tv.Get<DynamicNumber>(); dn.Set(n); tv.SetValue<DynamicNumber>(dn);
                    if (tv.AsString() == n.ToString()) return tv;
                }
            }
            catch (Exception e) { Write("number (method 2): " + e.Message); }
            if (NumCache.TryGetValue(n, out var c)) return TypedValue.CreateCopyFrom(c);
            throw new Exception("number " + n + " unavailable (change this setting once in FM)");
        }

        static readonly Dictionary<string, System.Type> s_managed = new();
        static System.Reflection.MethodInfo s_create;

        /// <summary>Typed value built exactly like the game does (Create&lt;T&gt; with the real type): needed for lists and references.</summary>
        internal static TypedValue TypedRef(Il2CppSystem.Object o)
        {
            try
            {
                string n = o.GetIl2CppType().FullName;
                if (!s_managed.TryGetValue(n, out var mt))
                {
                    if (o.TryCast<Il2CppSystem.Collections.Generic.List<TypedValue>>() != null) mt = typeof(Il2CppSystem.Collections.Generic.List<TypedValue>);
                    else foreach (var a in AppDomain.CurrentDomain.GetAssemblies()) { try { mt = a.GetType(n); } catch { } if (mt != null) break; }
                    s_managed[n] = mt;
                }
                if (mt != null)
                {
                    s_create ??= typeof(TypedValue).GetMethods().First(m => m.Name == "Create" && m.IsGenericMethodDefinition && m.GetParameters().Length == 1);
                    var typed = Activator.CreateInstance(mt, o.Pointer);
                    return (TypedValue)s_create.MakeGenericMethod(mt).Invoke(null, new[] { typed });
                }
            }
            catch (Exception e) { Write("typed value: " + (e.InnerException?.Message ?? e.Message)); }
            return RefValue(o);
        }

        internal static TypedValue RefValue(Il2CppSystem.Object o)
        {
            try { return TypedValue.Create<Il2CppSystem.Object>(o); }
            catch { var v = TypedValue.GetReferenceTypedValue(); v.SetValue(o); return v; }
        }

        internal static string Esc(string s)
        {
            if (s == null) return "";
            var sb = new StringBuilder(s.Length + 8);
            foreach (char ch in s)
            {
                if (ch == '\\') sb.Append("\\\\");
                else if (ch == '"') sb.Append("\\\"");
                else if (ch < ' ') sb.Append("\\u").Append(((int)ch).ToString("x4"));
                else sb.Append(ch);
            }
            return sb.ToString();
        }

        /// <summary>Order description for EVENTS (used to re-capture ids after a patch).</summary>
        internal static string Describe(uint ev, InteropReference r, TypedValue v)
        {
            var sb = new StringBuilder("{\"ev\":").Append(ev);
            try { sb.Append(",\"ref\":\"").Append(Esc(r?.GetIl2CppType()?.FullName)).Append("\",\"pairs\":\"").Append(Esc(PairsOf(r))).Append('"'); } catch { }
            try { sb.Append(",\"vtype\":\"").Append(Esc(v == null || v.IsNull ? "null" : v.DataType?.FullName)).Append('"'); } catch { }
            try { if (v != null && !v.IsNull) sb.Append(",\"rec\":").Append(DescribeValue(v, 0)); } catch (Exception e) { sb.Append(",\"recerr\":\"").Append(Esc(e.Message)).Append('"'); }
            return sb.Append('}').ToString();
        }

        /// <summary>Record ({"type","entries":[[key,type,value]…]}), list ([[type,value]…]), reference ({"type","pairs"}) or plain string.</summary>
        internal static string DescribeValue(TypedValue v, int depth)
        {
            if (v == null || v.IsNull) return "null";
            object o = v.Get();
            if (o is Il2CppSystem.Object io)
            {
                var dict = io.TryCast<Il2CppSystem.Collections.Generic.Dictionary<uint, TypedValue>>();
                if (dict != null && depth < 6)
                {
                    var sb = new StringBuilder("{\"type\":\"" + Esc(io.GetIl2CppType().FullName) + "\",\"entries\":[");
                    bool first = true;
                    var e = dict.GetEnumerator();
                    while (e.MoveNext())
                    {
                        var kv = e.Current;
                        sb.Append(first ? "" : ",").Append('[').Append(kv.Key).Append(",\"").Append(Esc(kv.Value == null || kv.Value.IsNull ? "null" : kv.Value.DataType?.FullName)).Append("\",").Append(DescribeValue(kv.Value, depth + 1)).Append(']');
                        first = false;
                    }
                    return sb.Append("]}").ToString();
                }
                var list = io.TryCast<Il2CppSystem.Collections.Generic.List<TypedValue>>();
                if (list != null && depth < 6)
                {
                    var sb = new StringBuilder("[");
                    for (int i = 0; i < list.Count && i < 400; i++)
                        sb.Append(i > 0 ? "," : "").Append("[\"").Append(Esc(list[i] == null || list[i].IsNull ? "null" : list[i].DataType?.FullName)).Append("\",").Append(DescribeValue(list[i], depth + 1)).Append(']');
                    return sb.Append(']').ToString();
                }
                var r = io.TryCast<InteropReference>();
                if (r != null)
                {
                    var sb = new StringBuilder("{\"type\":\"" + Esc(io.GetIl2CppType().FullName) + "\",\"pairs\":[");
                    var ids = r.m_id;
                    if (ids != null) for (int i = 0; i < ids.Count; i++) sb.Append(i > 0 ? "," : "").Append('[').Append(ids[i].ID).Append(',').Append(ids[i].Value).Append(']');
                    return sb.Append("]}").ToString();
                }
            }
            try
            {
                if (v.DataType?.FullName == "FM.GamePlugin.GameDate")
                {
                    var dt = FM.GamePlugin.GameDate.ToDateTime(v.Get<FM.GamePlugin.GameDate>());
                    return "\"" + dt.Year.ToString("0000") + "-" + dt.Month.ToString("00") + "-" + dt.Day.ToString("00") + "\"";
                }
            }
            catch { }
            return "\"" + Esc(v.AsString()) + "\"";
        }

        // ------------------------------------------------------------ JSON -> game value (same format as DescribeValue)
        internal static TypedValue FromJson(System.Text.Json.JsonElement e)
        {
            if (e.ValueKind == System.Text.Json.JsonValueKind.Object) return ObjectValue(e);
            if (e.ValueKind == System.Text.Json.JsonValueKind.Array)
            {
                if (e.GetArrayLength() == 0 || e[0].ValueKind == System.Text.Json.JsonValueKind.Array) return ListValue(e);
                return Leaf(e[0].GetString(), e[1]);
            }
            return TypedValue.Null;
        }

        static TypedValue ObjectValue(System.Text.Json.JsonElement e)
        {
            if (e.TryGetProperty("pairs", out var ps))
            {
                string pairs = string.Join(";", ps.EnumerateArray().Select(p => p[0].GetUInt32() + "=" + p[1].GetInt32()));
                return TypedRef(BuildRef(e.GetProperty("type").GetString(), pairs));
            }
            string type = e.GetProperty("type").GetString();
            Il2CppSystem.Object obj = type == "SI.Core.Record" ? new SI.Core.Record() : new SI.Bindable.DynamicReference();
            var dict = obj.Cast<Il2CppSystem.Collections.Generic.Dictionary<uint, TypedValue>>();
            foreach (var en in e.GetProperty("entries").EnumerateArray())
            {
                var val = en[2];
                dict[en[0].GetUInt32()] = val.ValueKind == System.Text.Json.JsonValueKind.Object ? ObjectValue(val)
                    : val.ValueKind == System.Text.Json.JsonValueKind.Array ? ListValue(val) : Leaf(en[1].GetString(), val);
            }
            return RefValue(obj);
        }

        static TypedValue ListValue(System.Text.Json.JsonElement e)
        {
            var list = new Il2CppSystem.Collections.Generic.List<TypedValue>();
            foreach (var it in e.EnumerateArray())
                list.Add(it[1].ValueKind == System.Text.Json.JsonValueKind.Object ? ObjectValue(it[1]) : Leaf(it[0].GetString(), it[1]));
            return TypedRef(list);
        }

        static TypedValue Leaf(string t, System.Text.Json.JsonElement v)
        {
            string s = v.ValueKind == System.Text.Json.JsonValueKind.String ? v.GetString() : v.GetRawText();
            var inv = System.Globalization.CultureInfo.InvariantCulture;
            switch (t)
            {
                case "null": return TypedValue.Null;
                case "SI.Bindable.NullReference": return RefValue(SI.Bindable.NullReference.Instance);
                case "SI.Core.DynamicNumber": return Number(int.Parse(s));
                case "System.String": return TypedValue.Create(s);
                case "System.Boolean": return TypedValue.Create(s.Equals("true", StringComparison.OrdinalIgnoreCase));
                case "System.Int32": return TypedValue.Create(int.Parse(s));
                case "System.UInt32": return TypedValue.Create(uint.Parse(s));
                case "System.Int64": return TypedValue.Create(long.Parse(s));
                case "System.Single": return TypedValue.Create(float.Parse(s, inv));
                case "System.Double": return TypedValue.Create(double.Parse(s, inv));
                case "FM.GamePlugin.GameDate":
                    var d = DateTime.Parse(s, inv);
                    return TypedValue.CreateValueType<FM.GamePlugin.GameDate>(new FM.GamePlugin.GameDate(new Il2CppSystem.DateTime(d.Year, d.Month, d.Day)));
                default: throw new Exception("unsupported value type: " + t);
            }
        }

        // ------------------------------------------------------------ commands
        internal static string SendJson(uint ev, string refType, string pairs, string json)
        {
            if (Interop == null) return "ERR FM not ready yet";
            var r = BuildRef(refType, pairs == "-" ? null : pairs);
            TypedValue v;
            using (var doc = System.Text.Json.JsonDocument.Parse(json)) v = FromJson(doc.RootElement);
            Replaying = true;
            try { Interop.SendEvent(ev, r, v); }
            finally { Replaying = false; }
            Write($"sent {ev} {refType} [{PairsOf(r)}]");
            return "OK";
        }

        internal static readonly Dictionary<ulong, string> Pending = new();
        static ulong NextKey = 0xFB29000000000001UL;

        internal static void OnData(ulong key, TypedValue v)
        {
            lock (Pending) if (!Pending.ContainsKey(key)) return;
            string d;
            try { d = DescribeValue(v, 0); } catch (Exception e) { d = "\"ERR " + Esc(e.Message) + "\""; }
            lock (Pending) Pending[key] = d;
        }

        internal static string Read(string[] reqs, int timeoutMs = 4000)
        {
            if (Interop == null) return "ERR FM not ready yet";
            var keys = new ulong[reqs.Length];
            var err = new string[reqs.Length];
            RunOnMain(() =>
            {
                for (int i = 0; i < reqs.Length; i++)
                {
                    try
                    {
                        var q = reqs[i].Split('|');
                        var r = BuildRef(q[0], q[1] == "-" ? null : q[1]);
                        ulong key = NextKey++;
                        lock (Pending) Pending[key] = null;
                        keys[i] = key;
                        var pid = new SI.Bindable.Reference.Core.PropertyID(Prop(q[2]));
                        var k = new SI.Bindable.Bindings.Key(key);
                        Interop.AddNode(k, SI.Bindable.Bindings.RootKey, pid);
                        Interop.OpenChannel(r, pid, k);
                    }
                    catch (Exception e) { err[i] = "ERR " + e.Message; }
                }
                return "OK";
            });
            var sw = System.Diagnostics.Stopwatch.StartNew();
            while (sw.ElapsedMilliseconds < timeoutMs)
            {
                bool all = true;
                lock (Pending) for (int i = 0; i < keys.Length; i++) if (keys[i] != 0 && Pending[keys[i]] == null) { all = false; break; }
                if (all) break;
                Thread.Sleep(30);
            }
            var res = new string[reqs.Length];
            lock (Pending)
                for (int i = 0; i < keys.Length; i++)
                {
                    if (keys[i] == 0) { res[i] = err[i]; continue; }
                    res[i] = Pending[keys[i]] ?? "ERR no reply";
                    Pending.Remove(keys[i]);
                }
            RunOnMain(() => { foreach (var k in keys) if (k != 0) try { var kk = new SI.Bindable.Bindings.Key(k); Interop.CloseChannel(kk); Interop.RemoveNode(kk); } catch { } return "OK"; });
            return string.Join("\u001e", res);
        }

        internal static string WriteChannel(string type, string pairs, string prop, string json)
        {
            if (Interop == null) return "ERR FM not ready yet";
            var r = BuildRef(type, pairs == "-" ? null : pairs);
            var pid = new SI.Bindable.Reference.Core.PropertyID(Prop(prop));
            var k = new SI.Bindable.Bindings.Key(NextKey++);
            Interop.AddNode(k, SI.Bindable.Bindings.RootKey, pid);
            Interop.OpenChannel(r, pid, k);
            TypedValue v;
            using (var doc = System.Text.Json.JsonDocument.Parse(json)) v = FromJson(doc.RootElement);
            Replaying = true;
            try { Interop.UpdateChannelData(k, v); }
            finally { Replaying = false; }
            // close the channel a bit later, once the engine has processed the write
            new Thread(() => { Thread.Sleep(3000); MainThread.Enqueue(() => { try { Interop.CloseChannel(k); Interop.RemoveNode(k); } catch { } }); }) { IsBackground = true }.Start();
            Write($"wrote {type} [{PairsOf(r)}] {prop}");
            return "OK";
        }

        static uint Prop(string s)
        {
            if (uint.TryParse(s, out var n)) return n;
            if (s.Length == 4) return (uint)(s[0] << 24 | s[1] << 16 | s[2] << 8 | s[3]);
            throw new Exception("invalid property: " + s);
        }

        static string FmVersion()
        {
            try { return Application.version; } catch { return "?"; }
        }

        static readonly object HandleLock = new();

        static string Handle(string cmd)
        {
            var parts = cmd.Split(' ', 2);
            switch (parts[0].ToUpperInvariant())
            {
                case "PING": return Interop != null ? "PONG ready" : "PONG waiting";
                case "VERSION": return "OK " + Version + " " + FmVersion();
                case "REFS": return "OK session=" + SessionId + " " + string.Join(" | ", PairsByType.Select(kv => kv.Key + "=" + kv.Value));
                case "EVENTS": lock (Events) { var s = string.Join("\u001e", Events); Events.Clear(); return "OK " + s; }
                case "RECORD": Recording = parts.Length > 1 && parts[1] == "1"; return "OK";
                case "READ": return "OK " + Read(parts[1].Split(";;", StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries));
                case "SENDJ":
                    var sj = parts[1].Split(' ', 4);
                    return RunOnMain(() => SendJson(uint.Parse(sj[0]), sj[1], sj[2], sj[3]));
                case "WRITE":
                    var wp = parts[1].Split(' ', 2);
                    var wq = wp[0].Split('|');
                    return RunOnMain(() => WriteChannel(wq[0], wq[1], wq[2], wp[1]));
                default: return "ERR unknown command";
            }
        }

        static string RunOnMain(Func<string> f, int timeoutMs = 10000)
        {
            string res = "ERR timeout";
            using var done = new ManualResetEventSlim();
            MainThread.Enqueue(() => { try { res = f(); } catch (Exception e) { res = "ERR " + e.Message; } finally { done.Set(); } });
            done.Wait(timeoutMs);
            return res;
        }

        // ------------------------------------------------------------ local server (loopback only)
        static void Server()
        {
            TcpListener l;
            try { l = new TcpListener(IPAddress.Loopback, Port); l.Start(); }
            catch (Exception e) { Write($"port {Port} unavailable: " + e.Message); return; }
            while (true)
            {
                try { var c = l.AcceptTcpClient(); new Thread(() => Client(c)) { IsBackground = true }.Start(); }
                catch (Exception e) { Write("server: " + e.Message); Thread.Sleep(500); }
            }
        }

        static void Client(TcpClient c)
        {
            try
            {
                using (c)
                using (var s = c.GetStream())
                using (var rd = new StreamReader(s, Encoding.UTF8))
                using (var wr = new StreamWriter(s, new UTF8Encoding(false)) { AutoFlush = true, NewLine = "\n" })
                {
                    string line;
                    while ((line = rd.ReadLine()) != null)
                    {
                        string reply;
                        try { lock (HandleLock) reply = Handle(line.Trim()); } catch (Exception e) { reply = "ERR " + e.Message; }
                        wr.WriteLine(reply.Replace("\r", " ").Replace("\n", " "));
                    }
                }
            }
            catch { }
        }
    }

    public class BridgeBehaviour : MonoBehaviour
    {
        public BridgeBehaviour(IntPtr p) : base(p) { }
        void Update()
        {
            int n = 0;
            while (n++ < 200 && Plugin.MainThread.TryDequeue(out var a)) a();
        }
    }

    static class DataPatches
    {
        static bool s_warned;

        [HarmonyPrefix]
        [HarmonyPatch(typeof(FM.UI.InteropDataHandler), nameof(FM.UI.InteropDataHandler.OnChannelDataChanged))]
        static void OnChannelDataChanged(Il2CppSystem.ReadOnlySpan<ulong> __0, Il2CppSystem.ReadOnlySpan<uint> __1, Il2CppSystem.Collections.Generic.List<TypedValue> __2, Il2CppSystem.ReadOnlySpan<long> __3)
        {
            try
            {
                if (__2 == null) return;
                var keys = __0.ToArray();
                int n = Math.Min(keys.Length, __2.Count);
                for (int i = 0; i < n; i++) Plugin.OnData(keys[i], __2[i]);
            }
            catch (Exception e) { if (!s_warned) { s_warned = true; Plugin.Write("data reading: " + e.Message); } }
        }
    }

    static class Patches
    {
        [HarmonyPrefix]
        [HarmonyPatch(typeof(GameInteropSubsystem), nameof(GameInteropSubsystem.OpenChannel))]
        static void OpenChannel(GameInteropSubsystem __instance, InteropReference __0)
        {
            Plugin.Interop ??= __instance;
            Plugin.Learn(__0);
        }

        [HarmonyPrefix]
        [HarmonyPatch(typeof(GameInteropSubsystem), nameof(GameInteropSubsystem.SendEvent))]
        static void SendEvent(GameInteropSubsystem __instance, uint __0, InteropReference __1, TypedValue __2)
        {
            Plugin.Interop ??= __instance;
            if (Plugin.Replaying) return;
            try
            {
                Plugin.Learn(__1);
                Plugin.LearnNumbers(__2);
                if (!Plugin.Recording) return;
                var d = Plugin.Describe(__0, __1, __2);
                lock (Plugin.Events) { Plugin.Events.Add(d); if (Plugin.Events.Count > 500) Plugin.Events.RemoveAt(0); }
            }
            catch (Exception e) { Plugin.Write("record error: " + e.Message); }
        }
    }
}
