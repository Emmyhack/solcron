import { PublicKey } from '@solana/web3.js';

// Trigger Types
export type TriggerType = 'time-based' | 'conditional' | 'log-based' | 'hybrid';

export interface TimeTrigger {
  type: 'time-based';
  interval: number; // seconds
}

export interface ConditionalTrigger {
  type: 'conditional';
  condition: string;
  parameters?: Record<string, any>;
}

export interface LogTrigger {
  type: 'log-based';
  eventSignature: string;
  sourceProgram?: string;
}

export interface HybridTrigger {
  type: 'hybrid';
  triggers: (TimeTrigger | ConditionalTrigger | LogTrigger)[];
}

export type Trigger = TimeTrigger | ConditionalTrigger | LogTrigger | HybridTrigger;

// Job Types
export interface AutomationJob {
  jobId: string;
  owner: string;
  targetProgram: string;
  targetInstruction: string;
  trigger: Trigger;
  gasLimit: number;
  balance: number; // in lamports
  minBalance: number; // in lamports
  isActive: boolean;
  executionCount: number;
  lastExecution: number; // timestamp
  createdAt: number; // timestamp
  updatedAt: number; // timestamp
}

export interface JobExecution {
  id: string;
  jobId: string;
  jobName: string;
  jobType: string;
  executionId: string;
  keeper: string;
  keeperAddress: string;
  timestamp: number;
  gasUsed: number;
  feeCharged: number;
  rewardPaid: number;
  success: boolean;
  errorMessage?: string;
  transactionSignature: string;
  transactionHash: string;
}

// Keeper Types
export interface Keeper {
  address: string;
  stakeAmount: number; // in lamports
  reputationScore: number; // 0-10000 (basis points)
  isActive: boolean;
  totalExecutions: number;
  successfulExecutions: number;
  totalEarnings: number; // in lamports
  pendingRewards: number; // in lamports
  lastExecutionTime: number; // timestamp
  registeredAt: number; // timestamp
}

export interface KeeperStats {
  successRate: number;
  avgExecutionTime: number;
  dailyEarnings: number;
  weeklyEarnings: number;
  monthlyEarnings: number;
}

// Registry Types
export interface RegistryState {
  totalJobs: number;
  activeJobs: number;
  totalKeepers: number;
  activeKeepers: number;
  totalExecutions: number;
  successfulExecutions: number;
  protocolRevenue: number; // in lamports
  baseFee: number; // in lamports
  protocolFeeBps: number; // basis points
  minStake: number; // in lamports
  nextJobId: number;
}

// Analytics Types
export interface SystemMetrics {
  timestamp: number;
  registryStats: RegistryState;
  jobStats: JobMetrics;
  keeperStats: KeeperMetrics;
  networkStats: NetworkMetrics;
}

export interface JobMetrics {
  avgBalance: number;
  totalBalance: number;
  lowBalanceCount: number;
  executionSuccessRate: number;
  avgExecutionTime: number;
  triggerTypeDistribution: Record<string, number>;
}

export interface KeeperMetrics {
  avgReputation: number;
  totalStake: number;
  avgStake: number;
  keeperUtilization: number;
  topPerformers: KeeperPerformance[];
}

export interface KeeperPerformance {
  keeper: string;
  reputationScore: number;
  successRate: number;
  totalExecutions: number;
  earnings: number;
}

export interface NetworkMetrics {
  avgTransactionTime: number;
  networkCongestion: number;
  gasPriceTrend: number;
}

// Dashboard State Types
export interface DashboardState {
  jobs: AutomationJob[];
  keepers: Keeper[];
  registry: RegistryState | null;
  metrics: SystemMetrics | null;
  recentExecutions: JobExecution[];
  loading: boolean;
  error: string | null;
  selectedJob: AutomationJob | null;
  selectedKeeper: Keeper | null;
}

// Form Types
export interface CreateJobForm {
  targetProgram: string;
  targetInstruction: string;
  triggerType: TriggerType;
  interval?: number;
  condition?: string;
  eventSignature?: string;
  gasLimit: number;
  initialBalance: number;
}

export interface UpdateJobForm {
  gasLimit?: number;
  additionalBalance?: number;
  isActive?: boolean;
}

export interface RegisterKeeperForm {
  stakeAmount: number;
}

// Wallet Types
export interface WalletState {
  connected: boolean;
  publicKey: PublicKey | null;
  connecting: boolean;
  balance: number; // in lamports
}

// API Response Types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

// UI Types
export type AlertType = 'success' | 'error' | 'warning' | 'info';

export interface Alert {
  id: string;
  type: AlertType;
  title: string;
  message: string;
  timestamp: number;
  dismissed?: boolean;
}

export interface TableColumn<T> {
  key: keyof T;
  title: string;
  render?: (value: any, item: T) => any;
  sortable?: boolean;
  width?: string;
}

export interface TableProps<T> {
  data: T[];
  columns: TableColumn<T>[];
  loading?: boolean;
  pagination?: {
    page: number;
    pageSize: number;
    total: number;
    onPageChange: (page: number) => void;
  };
  onRowClick?: (item: T) => void;
}

// Constants
export const TRIGGER_TYPES: { value: TriggerType; label: string; description: string }[] = [
  {
    value: 'time-based',
    label: 'Time-based',
    description: 'Execute at regular intervals'
  },
  {
    value: 'conditional',
    label: 'Conditional',
    description: 'Execute when conditions are met'
  },
  {
    value: 'log-based',
    label: 'Event-based',
    description: 'Execute on blockchain events'
  },
  {
    value: 'hybrid',
    label: 'Hybrid',
    description: 'Combine multiple trigger types'
  }
];

export const JOB_STATUS_COLORS = {
  active: 'bg-success-100 text-success-800 dark:bg-success-900 dark:text-success-200',
  inactive: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200',
  error: 'bg-error-100 text-error-800 dark:bg-error-900 dark:text-error-200'
} as const;

export const KEEPER_STATUS_COLORS = {
  active: 'text-success-600 dark:text-success-400',
  inactive: 'text-gray-600 dark:text-gray-400'
} as const;