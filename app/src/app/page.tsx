import { DashboardLayout } from '@/components/layout/DashboardLayout';
import { DashboardOverview } from '@/components/dashboard/DashboardOverview';

export default function HomePage() {
  return (
    <DashboardLayout>
      <div className="space-y-8">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold tracking-tight gradient-text">
              SolCron Dashboard
            </h1>
            <p className="text-muted-foreground mt-2">
              Monitor and manage your automation jobs on Solana
            </p>
          </div>
        </div>
        
        <DashboardOverview />
      </div>
    </DashboardLayout>
  );
}