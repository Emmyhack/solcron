'use client';

import React from 'react';
import { useDashboardStore } from '@/store/dashboard';
import { formatAddress, formatSOL, formatDuration, getJobStatusText, getJobStatusColor } from '@/lib/utils';
import { AutomationJob } from '@/types';

interface JobsTableProps {
  limit?: number;
}

export function JobsTable({ limit }: JobsTableProps) {
  const { jobs, loading } = useDashboardStore();
  
  const displayJobs = limit ? jobs.slice(0, limit) : jobs;

  if (loading) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="animate-pulse">
            <div className="h-12 bg-muted rounded"></div>
          </div>
        ))}
      </div>
    );
  }

  if (displayJobs.length === 0) {
    return (
      <div className="text-center py-8">
        <div className="text-4xl mb-2">🎯</div>
        <h3 className="text-lg font-medium mb-1">No jobs found</h3>
        <p className="text-muted-foreground text-sm">
          Create your first automation job to get started.
        </p>
      </div>
    );
  }

  const getStatusBadge = (job: AutomationJob) => {
    const statusColor = getJobStatusColor(job.isActive, job.balance < job.minBalance);
    const statusText = getJobStatusText(job.isActive, job.balance < job.minBalance);
    
    return (
      <span className={`job-status-badge job-status-${statusColor}`}>
        {statusText}
      </span>
    );
  };

  const getTriggerDescription = (job: AutomationJob) => {
    switch (job.trigger.type) {
      case 'time-based':
        return `Every ${formatDuration(job.trigger.interval)}`;
      case 'conditional':
        return job.trigger.condition || 'Conditional';
      case 'log-based':
        return 'Event-based';
      case 'hybrid':
        return 'Hybrid trigger';
      default:
        return 'Unknown';
    }
  };

  return (
    <div className="space-y-3">
      {displayJobs.map((job) => (
        <div
          key={job.jobId}
          className="border rounded-lg p-4 hover:bg-muted/50 transition-colors cursor-pointer"
        >
          <div className="flex items-start justify-between">
            <div className="flex-1 min-w-0">
              <div className="flex items-center space-x-2 mb-1">
                <span className="font-medium text-sm">
                  Job #{job.jobId}
                </span>
                {getStatusBadge(job)}
              </div>
              
              <div className="text-xs text-muted-foreground space-y-1">
                <div>Target: {formatAddress(job.targetProgram)}</div>
                <div>Instruction: {job.targetInstruction}</div>
                <div>Trigger: {getTriggerDescription(job)}</div>
              </div>
            </div>
            
            <div className="text-right text-xs space-y-1">
              <div className="font-medium">
                {formatSOL(job.balance)} SOL
              </div>
              <div className="text-muted-foreground">
                {job.executionCount} executions
              </div>
              {job.lastExecution > 0 && (
                <div className="text-muted-foreground">
                  Last: {formatDuration(Math.floor(Date.now() / 1000) - job.lastExecution)} ago
                </div>
              )}
            </div>
          </div>
          
          {/* Progress bar for balance */}
          <div className="mt-3">
            <div className="w-full bg-muted rounded-full h-1">
              <div
                className={`h-1 rounded-full transition-all duration-300 ${
                  job.balance < job.minBalance 
                    ? 'bg-error-500' 
                    : job.balance < job.minBalance * 5 
                    ? 'bg-warning-500' 
                    : 'bg-success-500'
                }`}
                style={{
                  width: `${Math.min(100, (job.balance / (job.minBalance * 10)) * 100)}%`
                }}
              />
            </div>
          </div>
        </div>
      ))}
      
      {limit && jobs.length > limit && (
        <div className="text-center pt-2">
          <button className="text-sm text-primary hover:text-primary/80 transition-colors">
            View all {jobs.length} jobs →
          </button>
        </div>
      )}
    </div>
  );
}