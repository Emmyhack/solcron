import '@/styles/globals.css';
import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import { WalletContextProvider } from '@/components/providers/WalletProvider';
import { SolCronProvider } from '@/components/providers/SolCronProvider';
import { DashboardProvider } from '@/components/providers/DashboardProvider';
import { ToastProvider } from '@/components/providers/ToastProvider';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'SolCron - Solana Automation Platform',
  description: 'Professional decentralized automation platform for Solana blockchain',
  keywords: ['Solana', 'DeFi', 'Automation', 'Smart Contracts', 'Blockchain', 'Chainlink'],
  authors: [{ name: 'SolCron Team' }],
  viewport: 'width=device-width, initial-scale=1',
  themeColor: '#3b82f6',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="h-full">
      <body className={`${inter.className} h-full bg-slate-50 dark:bg-slate-900 text-slate-900 dark:text-slate-100 antialiased`}>
        <WalletContextProvider>
          <SolCronProvider>
            <DashboardProvider>
              <ToastProvider>
                <div className="min-h-screen flex flex-col bg-slate-50 dark:bg-slate-900">
                  {children}
                </div>
              </ToastProvider>
            </DashboardProvider>
          </SolCronProvider>
        </WalletContextProvider>
      </body>
    </html>
  );
}