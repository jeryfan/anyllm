import type { ReactNode } from "react";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

type StatusType = "success" | "warning" | "error" | "info" | "neutral";

const statusStyles: Record<StatusType, string> = {
  success: "bg-emerald-500/10 text-emerald-600 border-emerald-500/20 dark:text-emerald-400",
  warning: "bg-amber-500/10 text-amber-600 border-amber-500/20 dark:text-amber-400",
  error: "bg-red-500/10 text-red-600 border-red-500/20 dark:text-red-400",
  info: "bg-blue-500/10 text-blue-600 border-blue-500/20 dark:text-blue-400",
  neutral: "bg-muted text-muted-foreground",
};

interface StatusBadgeProps {
  status: StatusType;
  children: ReactNode;
  className?: string;
}

export function StatusBadge({ status, children, className }: StatusBadgeProps) {
  return (
    <Badge className={cn(statusStyles[status], className)}>
      {children}
    </Badge>
  );
}

export function HttpStatusBadge({ status }: { status: number | null }) {
  if (status === null || status === undefined) {
    return <StatusBadge status="neutral">N/A</StatusBadge>;
  }
  if (status >= 200 && status < 300) {
    return <StatusBadge status="success">{status}</StatusBadge>;
  }
  if (status >= 300 && status < 400) {
    return <StatusBadge status="info">{status}</StatusBadge>;
  }
  if (status >= 400 && status < 500) {
    return <StatusBadge status="warning">{status}</StatusBadge>;
  }
  return <StatusBadge status="error">{status}</StatusBadge>;
}

export function EnabledBadge({ enabled, enabledText, disabledText }: { enabled: boolean; enabledText: string; disabledText: string }) {
  return enabled
    ? <StatusBadge status="success">{enabledText}</StatusBadge>
    : <StatusBadge status="neutral">{disabledText}</StatusBadge>;
}

const methodColors: Record<string, StatusType> = {
  GET: "info",
  POST: "success",
  PUT: "warning",
  DELETE: "error",
  PATCH: "info",
};

export function MethodBadge({ method }: { method: string }) {
  const upper = method.toUpperCase();
  return <StatusBadge status={methodColors[upper] ?? "neutral"}>{upper}</StatusBadge>;
}
