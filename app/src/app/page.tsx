import { DashboardLayout } from '@/components/layout/DashboardLayout';
import { DashboardOverview } from '@/components/dashboard/DashboardOverview';

export default function HomePage() {
  return (
    <DashboardLayout>
      <div className="space-y-8">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-semibold tracking-tight text-slate-900 dark:text-white">
              SolCron Dashboard
            </h1>
            <p className="text-slate-600 dark:text-slate-300 mt-1">
              Monitor and manage your automation jobs on Solana blockchain
            </p>
          </div>
        </div>
        
        <DashboardOverview />
      </div>
    </DashboardLayout>
  );
}