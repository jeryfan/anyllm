import { useEffect, useState } from "react";
import {
  getConfig,
  listChannels,
  listTokens,
  listModelMappings,
  getUsageStats,
} from "@/lib/tauri";
import type {
  AppConfig,
  Channel,
  Token,
  ModelMapping,
  UsageStats,
} from "@/lib/tauri";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Activity,
  Network,
  KeyRound,
  ArrowRightLeft,
} from "lucide-react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip as RechartsTooltip,
  ResponsiveContainer,
  CartesianGrid,
} from "recharts";
import { useLanguage } from "@/lib/i18n";

interface DashboardData {
  config: AppConfig;
  channels: Channel[];
  tokens: Token[];
  modelMappings: ModelMapping[];
  usageStats: UsageStats;
}

function StatCardSkeleton() {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="h-4 w-24 animate-pulse rounded bg-muted" />
          <div className="h-5 w-5 animate-pulse rounded bg-muted" />
        </div>
      </CardHeader>
      <CardContent>
        <div className="h-8 w-16 animate-pulse rounded bg-muted" />
      </CardContent>
    </Card>
  );
}

function ChartSkeleton() {
  return (
    <Card>
      <CardHeader>
        <div className="h-5 w-40 animate-pulse rounded bg-muted" />
        <div className="h-4 w-56 animate-pulse rounded bg-muted" />
      </CardHeader>
      <CardContent>
        <div className="h-[300px] w-full animate-pulse rounded bg-muted" />
      </CardContent>
    </Card>
  );
}

export default function Dashboard() {
  const { t } = useLanguage();
  const [data, setData] = useState<DashboardData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      getConfig(),
      listChannels(),
      listTokens(),
      listModelMappings(),
      getUsageStats(7),
    ])
      .then(([config, channels, tokens, modelMappings, usageStats]) => {
        setData({ config, channels, tokens, modelMappings, usageStats });
      })
      .catch((err) => {
        setError(String(err));
      });
  }, []);

  const totalRequests = data
    ? data.usageStats.daily.reduce((sum, d) => sum + d.count, 0)
    : 0;
  const activeChannels = data
    ? data.channels.filter((c) => c.enabled).length
    : 0;
  const activeTokens = data
    ? data.tokens.filter((tk) => tk.enabled).length
    : 0;
  const totalModels = data ? data.modelMappings.length : 0;

  const statCards = [
    {
      title: t.dashboard.totalRequests,
      value: totalRequests,
      icon: Activity,
      description: t.dashboard.lastNDays(7),
    },
    {
      title: t.dashboard.activeChannels,
      value: activeChannels,
      icon: Network,
      description: t.dashboard.enabledChannels,
    },
    {
      title: t.dashboard.activeTokens,
      value: activeTokens,
      icon: KeyRound,
      description: t.dashboard.enabledTokens,
    },
    {
      title: t.dashboard.totalModels,
      value: totalModels,
      icon: ArrowRightLeft,
      description: t.dashboard.modelMappingsCount,
    },
  ];

  const chartData = data
    ? data.usageStats.daily.map((d) => ({
        date: d.date.slice(5), // "MM-DD" format
        requests: d.count,
        tokens: d.prompt_tokens + d.completion_tokens,
      }))
    : [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold">{t.dashboard.title}</h1>
        <p className="mt-1 text-muted-foreground">
          {t.dashboard.subtitle}
        </p>
      </div>

      {/* Error state */}
      {error && (
        <Card className="border-destructive">
          <CardContent className="pt-0">
            <p className="text-sm text-destructive">
              {t.dashboard.failedToLoad} {error}
            </p>
          </CardContent>
        </Card>
      )}

      {/* Stat cards */}
      <div className="grid gap-4 md:grid-cols-4">
        {!data
          ? Array.from({ length: 4 }).map((_, i) => <StatCardSkeleton key={i} />)
          : statCards.map((stat) => {
              const Icon = stat.icon;
              return (
                <Card key={stat.title}>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <CardDescription>{stat.title}</CardDescription>
                      <Icon className="h-5 w-5 text-muted-foreground" />
                    </div>
                  </CardHeader>
                  <CardContent>
                    <div className="text-3xl font-bold">{stat.value.toLocaleString()}</div>
                    <p className="mt-1 text-xs text-muted-foreground">{stat.description}</p>
                  </CardContent>
                </Card>
              );
            })}
      </div>

      {/* Request trend chart */}
      {!data ? (
        <ChartSkeleton />
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>{t.dashboard.requestTrend}</CardTitle>
            <CardDescription>{t.dashboard.requestTrendDesc}</CardDescription>
          </CardHeader>
          <CardContent>
            {chartData.length === 0 ? (
              <div className="flex h-[300px] items-center justify-center text-muted-foreground">
                {t.dashboard.noRequestData}
              </div>
            ) : (
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={chartData}>
                  <defs>
                    <linearGradient id="gradientRequests" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="0%" stopColor="hsl(221 83% 53%)" stopOpacity={0.3} />
                      <stop offset="100%" stopColor="hsl(221 83% 53%)" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" vertical={false} stroke="hsl(var(--border))" />
                  <XAxis
                    dataKey="date"
                    tickLine={false}
                    axisLine={false}
                    fontSize={12}
                    stroke="hsl(var(--muted-foreground))"
                  />
                  <YAxis
                    tickLine={false}
                    axisLine={false}
                    fontSize={12}
                    allowDecimals={false}
                    stroke="hsl(var(--muted-foreground))"
                  />
                  <RechartsTooltip
                    contentStyle={{
                      borderRadius: "8px",
                      border: "1px solid hsl(var(--border))",
                      backgroundColor: "hsl(var(--popover))",
                      color: "hsl(var(--popover-foreground))",
                      fontSize: "13px",
                    }}
                  />
                  <Area
                    type="monotone"
                    dataKey="requests"
                    name="Requests"
                    stroke="hsl(221 83% 53%)"
                    strokeWidth={2}
                    fill="url(#gradientRequests)"
                    dot={{ r: 3, fill: "hsl(221 83% 53%)", strokeWidth: 0 }}
                    activeDot={{ r: 5, fill: "hsl(221 83% 53%)", strokeWidth: 2, stroke: "#fff" }}
                  />
                </AreaChart>
              </ResponsiveContainer>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
