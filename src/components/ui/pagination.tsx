import { ChevronLeft, ChevronRight, MoreHorizontal } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "./button";

interface PaginationProps {
  current: number;
  total: number;
  pageSize: number;
  onChange: (page: number) => void;
  className?: string;
}

function getPageNumbers(current: number, totalPages: number): (number | "ellipsis")[] {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, i) => i + 1);
  }

  const pages: (number | "ellipsis")[] = [1];

  if (current > 3) {
    pages.push("ellipsis");
  }

  const start = Math.max(2, current - 1);
  const end = Math.min(totalPages - 1, current + 1);

  for (let i = start; i <= end; i++) {
    pages.push(i);
  }

  if (current < totalPages - 2) {
    pages.push("ellipsis");
  }

  pages.push(totalPages);

  return pages;
}

export function Pagination({ current, total, pageSize, onChange, className }: PaginationProps) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const pages = getPageNumbers(current, totalPages);

  return (
    <div className={cn("flex items-center justify-between", className)}>
      <p className="text-sm text-muted-foreground tabular-nums">
        {total} 条
      </p>
      <div className="flex items-center gap-1">
        <Button
          variant="outline"
          size="icon-xs"
          disabled={current <= 1}
          onClick={() => onChange(current - 1)}
        >
          <ChevronLeft />
        </Button>
        {pages.map((page, i) =>
          page === "ellipsis" ? (
            <span key={`e${i}`} className="flex size-6 items-center justify-center">
              <MoreHorizontal className="size-3 text-muted-foreground" />
            </span>
          ) : (
            <Button
              key={page}
              variant={page === current ? "default" : "outline"}
              size="icon-xs"
              onClick={() => onChange(page)}
            >
              {page}
            </Button>
          ),
        )}
        <Button
          variant="outline"
          size="icon-xs"
          disabled={current >= totalPages}
          onClick={() => onChange(current + 1)}
        >
          <ChevronRight />
        </Button>
      </div>
    </div>
  );
}
