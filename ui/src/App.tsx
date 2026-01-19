import { useEffect, useState } from 'react';
import axios from 'axios';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { LayoutDashboard, Server, Settings, Activity, HardDrive, LogOut } from 'lucide-react';

interface SystemMetrics {
  cpu_usage: number;
  total_memory: number;
  used_memory: number;
  memory_percentage: number;
  total_disk: number;
  used_disk: number;
  disk_percentage: number;
  os_name: string;
  host_name: string;
}

interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory: number;
}

function App() {
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');

  // Tabs: 'dashboard' | 'processes'
  const [activeTab, setActiveTab] = useState('dashboard');

  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [history, setHistory] = useState<{ time: string; cpu: number; mem: number }[]>([]);

  // Configure Axios Auth Header
  useEffect(() => {
    if (token) {
      axios.defaults.headers.common['Authorization'] = `Bearer ${token}`;
    } else {
      delete axios.defaults.headers.common['Authorization'];
    }
  }, [token]);

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      const res = await axios.post('http://localhost:3000/api/login', { username, password });
      const newToken = res.data.token;
      localStorage.setItem('token', newToken);
      setToken(newToken);
      setError('');
    } catch (err) {
      setError('Invalid credentials');
    }
  };

  const handleLogout = () => {
    localStorage.removeItem('token');
    setToken(null);
    setMetrics(null);
  };

  const fetchData = async () => {
    if (!token) return;
    try {
      // Always fetch metrics for the header
      const sysRes = await axios.get('http://localhost:3000/api/system');
      const data = sysRes.data;
      setMetrics(data);

      setHistory(prev => {
        const now = new Date().toLocaleTimeString();
        const newEntry = { time: now, cpu: data.cpu_usage, mem: data.memory_percentage };
        const newHistory = [...prev, newEntry];
        if (newHistory.length > 20) newHistory.shift();
        return newHistory;
      });

      // Fetch processes only if tab is active
      if (activeTab === 'processes') {
        const procRes = await axios.get('http://localhost:3000/api/processes');
        setProcesses(procRes.data);
      }

    } catch (error) {
      console.error("Error fetching data:", error);
      if (axios.isAxiosError(error) && error.response?.status === 401) {
        handleLogout();
      }
    }
  };

  useEffect(() => {
    if (token) {
      fetchData();
      const interval = setInterval(fetchData, 2000);
      return () => clearInterval(interval);
    }
  }, [token, activeTab]);

  const formatBytes = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(2)} GB`;
  };

  const formatMb = (bytes: number) => {
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(0)} MB`;
  };

  if (!token) {
    return (
      <div className="flex h-screen items-center justify-center bg-gray-900 text-white">
        <div className="w-full max-w-md p-8 bg-gray-800 rounded-lg shadow-2xl border border-gray-700">
          <div className="flex justify-center mb-6">
            <div className="p-4 bg-blue-600 rounded-full">
              <Server size={40} />
            </div>
          </div>
          <h2 className="text-2xl font-bold text-center mb-6">RustPanel Login</h2>
          {error && <div className="bg-red-500/20 text-red-400 p-3 rounded mb-4 text-center">{error}</div>}
          <form onSubmit={handleLogin} className="space-y-4">
            <div>
              <label className="block text-sm text-gray-400 mb-1">Username</label>
              <input 
                type="text" 
                value={username}
                onChange={e => setUsername(e.target.value)}
                className="w-full p-3 bg-gray-700 rounded border border-gray-600 focus:border-blue-500 outline-none"
                placeholder="admin"
              />
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">Password</label>
              <input 
                type="password" 
                value={password}
                onChange={e => setPassword(e.target.value)}
                className="w-full p-3 bg-gray-700 rounded border border-gray-600 focus:border-blue-500 outline-none"
                placeholder="password"
              />
            </div>
            <button className="w-full bg-blue-600 hover:bg-blue-700 p-3 rounded font-bold transition-colors">
              Sign In
            </button>
          </form>
          <div className="mt-6 text-center text-xs text-gray-500">
            Default: admin / password
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-gray-900 text-white">
      {/* Sidebar */}
      <aside className="w-64 bg-gray-800 p-6 flex flex-col border-r border-gray-700">
        <h1 className="text-2xl font-bold mb-8 text-blue-400 flex items-center gap-2">
          <Server /> RustPanel
        </h1>
        <nav className="flex-1 space-y-2">
          <NavItem 
            icon={<LayoutDashboard />} 
            label="Dashboard" 
            active={activeTab === 'dashboard'} 
            onClick={() => setActiveTab('dashboard')}
          />
          <NavItem 
            icon={<Activity />} 
            label="Processes" 
            active={activeTab === 'processes'}
            onClick={() => setActiveTab('processes')}
          />
          <NavItem icon={<HardDrive />} label="Files" onClick={() => alert("Files coming soon!")} />
          <NavItem icon={<Settings />} label="Settings" onClick={() => alert("Settings coming soon!")} />
        </nav>
        <button onClick={handleLogout} className="flex items-center gap-3 p-3 rounded-lg text-red-400 hover:bg-red-500/10 cursor-pointer mt-auto">
          <LogOut size={20} />
          <span className="font-medium">Logout</span>
        </button>
      </aside>

      {/* Main Content */}
      <main className="flex-1 p-8 overflow-auto">
        <header className="mb-8 flex justify-between items-center">
          <div>
            <h2 className="text-3xl font-bold capitalize">{activeTab}</h2>
            <p className="text-gray-400">
              {metrics ? `${metrics.host_name} (${metrics.os_name})` : 'Connecting...'}
            </p>
          </div>
          <div className="flex items-center gap-2 bg-gray-800 px-4 py-2 rounded-full border border-gray-700">
            <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
            <span className="text-sm font-mono text-green-400">System Online</span>
          </div>
        </header>

        {activeTab === 'dashboard' && (
          <>
            {/* Stats Grid */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
              <StatCard 
                title="CPU Usage" 
                value={metrics ? `${metrics.cpu_usage.toFixed(1)}%` : '-'} 
                color="text-blue-400"
              />
              <StatCard 
                title="Memory Usage" 
                value={metrics ? `${metrics.memory_percentage.toFixed(1)}%` : '-'} 
                subValue={metrics ? `${formatBytes(metrics.used_memory)} / ${formatBytes(metrics.total_memory)}` : ''}
                color="text-purple-400"
              />
              <StatCard 
                title="Disk Usage" 
                value={metrics ? `${metrics.disk_percentage.toFixed(1)}%` : '-'} 
                subValue={metrics ? `${formatBytes(metrics.used_disk)} / ${formatBytes(metrics.total_disk)}` : ''}
                color="text-green-400"
              />
            </div>

            {/* Charts */}
            <div className="bg-gray-800 p-6 rounded-lg shadow-lg border border-gray-700">
              <h3 className="text-xl font-semibold mb-4 flex items-center gap-2">
                <Activity size={20} className="text-blue-400"/> Performance History
              </h3>
              <div className="h-64 w-full">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={history}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                    <XAxis dataKey="time" stroke="#9CA3AF" />
                    <YAxis stroke="#9CA3AF" />
                    <Tooltip 
                      contentStyle={{ backgroundColor: '#1F2937', border: '1px solid #374151', borderRadius: '0.5rem' }}
                      itemStyle={{ color: '#E5E7EB' }}
                    />
                    <Line type="monotone" dataKey="cpu" stroke="#60A5FA" strokeWidth={2} name="CPU %" dot={false} />
                    <Line type="monotone" dataKey="mem" stroke="#C084FC" strokeWidth={2} name="Memory %" dot={false} />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </div>
          </>
        )}

        {activeTab === 'processes' && (
          <div className="bg-gray-800 rounded-lg shadow-lg border border-gray-700 overflow-hidden">
            <div className="p-4 border-b border-gray-700 bg-gray-800/50">
              <h3 className="font-semibold text-lg flex items-center gap-2">
                <Activity size={20} className="text-purple-400"/> Top Processes
              </h3>
            </div>
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="bg-gray-900/50 text-gray-400 text-sm">
                  <th className="p-4 font-medium">PID</th>
                  <th className="p-4 font-medium">Name</th>
                  <th className="p-4 font-medium">CPU %</th>
                  <th className="p-4 font-medium">Memory</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-700">
                {processes.map((proc) => (
                  <tr key={proc.pid} className="hover:bg-gray-700/50 transition-colors">
                    <td className="p-4 font-mono text-gray-500">{proc.pid}</td>
                    <td className="p-4 font-medium text-white">{proc.name}</td>
                    <td className="p-4">
                      <div className="flex items-center gap-2">
                        <div className="w-24 h-2 bg-gray-700 rounded-full overflow-hidden">
                          <div 
                            className="h-full bg-blue-500 rounded-full" 
                            style={{ width: `${Math.min(proc.cpu_usage, 100)}%` }}
                          ></div>
                        </div>
                        <span className="text-sm">{proc.cpu_usage.toFixed(1)}%</span>
                      </div>
                    </td>
                    <td className="p-4 text-gray-400">{formatMb(proc.memory)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </main>
    </div>
  );
}

const NavItem = ({ icon, label, active = false, onClick }: { icon: any, label: string, active?: boolean, onClick?: () => void }) => (
  <div 
    onClick={onClick}
    className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-all ${active ? 'bg-blue-600 text-white shadow-lg shadow-blue-900/50' : 'text-gray-400 hover:bg-gray-700 hover:text-white'}`}
  >
    {icon}
    <span className="font-medium">{label}</span>
  </div>
);

const StatCard = ({ title, value, subValue, color }: { title: string, value: string, subValue?: string, color: string }) => (
  <div className="bg-gray-800 p-6 rounded-lg shadow-lg border-l-4 border-gray-700 hover:border-blue-500 transition-colors group">
    <h3 className="text-gray-400 font-medium mb-1 group-hover:text-white transition-colors">{title}</h3>
    <div className={`text-3xl font-bold ${color}`}>{value}</div>
    {subValue && <div className="text-sm text-gray-500 mt-1">{subValue}</div>}
  </div>
);

export default App;