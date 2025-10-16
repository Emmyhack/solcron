import '@/styles/globals.css';
import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import { WalletContextProvider } from '@/components/providers/WalletProvider';
import { SolCronProvider } from '@/components/providers/SolCronProvider';
import { DashboardProvider } from '@/components/providers/DashboardProvider';
import { ToastProvider } from '@/components/providers/ToastProvider';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'SolCron Dashboard',
  description: 'Decentralized automation platform for Solana',
  keywords: ['Solana', 'DeFi', 'Automation', 'Smart Contracts', 'Blockchain'],
  authors: [{ name: 'SolCron Team' }],
  viewport: 'width=device-width, initial-scale=1',
  themeColor: '#0ea5e9',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="h-full">
      <body className={`${inter.className} h-full bg-background`}>
        <WalletContextProvider>
          <SolCronProvider>
            <DashboardProvider>
              <ToastProvider>
                <div className="min-h-screen flex flex-col">
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