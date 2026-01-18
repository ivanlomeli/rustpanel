import { useEffect, useState } from 'react';
import axios from 'axios';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { LayoutDashboard, Server, Settings, Activity, HardDrive } from 'lucide-react';

interface SystemMetrics {
  cpu_usage: number;
  total_memory: number;
  used_memory: number;
  memory_percentage: number;
  os_name: string;
  host_name: string;
}

function App() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [history, setHistory] = useState<{ time: string; cpu: number; mem: number }[]>([]);

  const fetchMetrics = async () => {
    try {
      // In development, we point to the Rust server port
      const response = await axios.get('http://localhost:3000/api/system');
      const data = response.data;
      setMetrics(data);

      setHistory(prev => {
        const now = new Date().toLocaleTimeString();
        const newEntry = { time: now, cpu: data.cpu_usage, mem: data.memory_percentage };
        const newHistory = [...prev, newEntry];
        if (newHistory.length > 20) newHistory.shift();
        return newHistory;
      });
    } catch (error) {
      console.error("Error fetching metrics:", error);
    }
  };

  useEffect(() => {
    fetchMetrics();
    const interval = setInterval(fetchMetrics, 2000);
    return () => clearInterval(interval);
  }, []);

  const formatBytes = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(2)} GB`;
  };

  return (
    <div className="flex h-screen bg-gray-900 text-white">
      {/* Sidebar */}
      <aside className="w-64 bg-gray-800 p-6 flex flex-col">
        <h1 className="text-2xl font-bold mb-8 text-blue-400 flex items-center gap-2">
          <Server /> RustPanel
        </h1>
        <nav className="flex-1 space-y-2">
          <NavItem icon={<LayoutDashboard />} label="Dashboard" active />
          <NavItem icon={<HardDrive />} label="Files" />
          <NavItem icon={<Activity />} label="Processes" />
          <NavItem icon={<Settings />} label="Settings" />
        </nav>
        <div className="text-xs text-gray-500 mt-auto">
          v0.1.0 Beta
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 p-8 overflow-auto">
        <header className="mb-8">
          <h2 className="text-3xl font-bold">Server Overview</h2>
          <p className="text-gray-400">
            {metrics ? `${metrics.host_name} (${metrics.os_name})` : 'Connecting...'}
          </p>
        </header>

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
            title="Status" 
            value="Online" 
            color="text-green-400"
          />
        </div>

        {/* Charts */}
        <div className="bg-gray-800 p-6 rounded-lg shadow-lg">
          <h3 className="text-xl font-semibold mb-4">Performance History</h3>
          <div className="h-64 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={history}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="time" stroke="#9CA3AF" />
                <YAxis stroke="#9CA3AF" />
                <Tooltip 
                  contentStyle={{ backgroundColor: '#1F2937', border: 'none' }}
                  itemStyle={{ color: '#E5E7EB' }}
                />
                <Line type="monotone" dataKey="cpu" stroke="#60A5FA" strokeWidth={2} name="CPU %" />
                <Line type="monotone" dataKey="mem" stroke="#C084FC" strokeWidth={2} name="Memory %" />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </main>
    </div>
  );
}

const NavItem = ({ icon, label, active = false }: { icon: any, label: string, active?: boolean }) => (
  <div className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-colors ${active ? 'bg-blue-600 text-white' : 'text-gray-400 hover:bg-gray-700 hover:text-white'}`}>
    {icon}
    <span className="font-medium">{label}</span>
  </div>
);

const StatCard = ({ title, value, subValue, color }: { title: string, value: string, subValue?: string, color: string }) => (
  <div className="bg-gray-800 p-6 rounded-lg shadow-lg border-l-4 border-blue-500">
    <h3 className="text-gray-400 font-medium mb-1">{title}</h3>
    <div className={`text-3xl font-bold ${color}`}>{value}</div>
    {subValue && <div className="text-sm text-gray-500 mt-1">{subValue}</div>}
  </div>
);

export default App;