import type { ReactNode } from "react";
import { Loader2 } from "lucide-react";
import {
  Table,
  TableHeader,
  TableBody,
  TableHead,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";

interface DataTableProps {
  headers: { label: string; className?: string }[];
  loading?: boolean;
  loadingText?: string;
  children: ReactNode;
  className?: string;
}

export function DataTable({ headers, loading, loadingText, children, className }: DataTableProps) {
  if (loading) {
    return (
      <div className="flex items-center justify-center py-16">
        <Loader2 className="size-5 animate-spin text-muted-foreground" />
        {loadingText && (
          <span className="ml-2 text-sm text-muted-foreground">{loadingText}</span>
        )}
      </div>
    );
  }

  return (
    <div className={cn("table-wrapper", className)}>
      <Table>
        <TableHeader>
          <TableRow>
            {headers.map((h, i) => (
              <TableHead key={i} className={h.className}>{h.label}</TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {children}
        </TableBody>
      </Table>
    </div>
  );
}
