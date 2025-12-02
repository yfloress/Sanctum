import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
} from "recharts";
import { CHART_COLORS } from "../../types";

interface AnalyticsViewProps {
  expensesByCategory: { name: string; value: number }[];
  balanceEvolution: {
    date: string;
    balance: number;
    income: number;
    expense: number;
  }[];
  hasTransactions: boolean;
}

export function AnalyticsView({
  expensesByCategory,
  balanceEvolution,
  hasTransactions,
}: AnalyticsViewProps) {
  if (!hasTransactions) {
    return (
      <div className="analytics-page">
        <h1 className="page-title">Analytics</h1>
        <p className="empty-state">
          No transaction data available for analysis
        </p>
      </div>
    );
  }

  return (
    <div className="analytics-page">
      <h1 className="page-title">Analytics</h1>

      <div className="analytics-grid">
        <div className="chart-card">
          <h2 className="section-title">Expenses by Category</h2>
          {expensesByCategory.length === 0 ? (
            <p className="empty-state">No expenses recorded</p>
          ) : (
            <div className="chart-container">
              <ResponsiveContainer width="100%" height={300}>
                <PieChart>
                  <Pie
                    data={expensesByCategory}
                    cx="50%"
                    cy="50%"
                    innerRadius={60}
                    outerRadius={100}
                    paddingAngle={3}
                    dataKey="value"
                    stroke="none"
                  >
                    {expensesByCategory.map((_, index) => (
                      <Cell
                        key={`cell-${index}`}
                        fill={CHART_COLORS[index % CHART_COLORS.length]}
                      />
                    ))}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "#111827",
                      border: "1px solid #8b5cf6",
                      borderRadius: "8px",
                      color: "#e8ecf6",
                    }}
                    formatter={(value: number) => [
                      `$${value.toFixed(2)}`,
                      "Amount",
                    ]}
                  />
                </PieChart>
              </ResponsiveContainer>
              <div className="chart-legend">
                {expensesByCategory.map((entry, index) => (
                  <div key={entry.name} className="legend-item">
                    <span
                      className="legend-color"
                      style={{
                        backgroundColor:
                          CHART_COLORS[index % CHART_COLORS.length],
                      }}
                    />
                    <span className="legend-label">{entry.name}</span>
                    <span className="legend-value">
                      ${entry.value.toFixed(2)}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="chart-card">
          <h2 className="section-title">Balance Evolution</h2>
          <div className="chart-container">
            <ResponsiveContainer width="100%" height={300}>
              <AreaChart data={balanceEvolution}>
                <defs>
                  <linearGradient
                    id="balanceGradient"
                    x1="0"
                    y1="0"
                    x2="0"
                    y2="1"
                  >
                    <stop
                      offset="5%"
                      stopColor="#7f8aff"
                      stopOpacity={0.4}
                    />
                    <stop
                      offset="95%"
                      stopColor="#7f8aff"
                      stopOpacity={0}
                    />
                  </linearGradient>
                </defs>
                <CartesianGrid
                  stroke="#374151"
                  strokeDasharray="3 3"
                  vertical={false}
                />
                <XAxis
                  dataKey="date"
                  stroke="#8c93a8"
                  fontSize={12}
                  tickLine={false}
                  axisLine={{ stroke: "#374151" }}
                />
                <YAxis
                  stroke="#8c93a8"
                  fontSize={12}
                  tickLine={false}
                  axisLine={{ stroke: "#374151" }}
                  tickFormatter={(value) => `$${value}`}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#111827",
                    border: "1px solid #7f8aff",
                    borderRadius: "8px",
                    color: "#e8ecf6",
                  }}
                  formatter={(value: number) => [
                    `$${value.toFixed(2)}`,
                    "Balance",
                  ]}
                  labelStyle={{ color: "#c1c7d7" }}
                />
                <Area
                  type="monotone"
                  dataKey="balance"
                  stroke="#7f8aff"
                  strokeWidth={2}
                  fill="url(#balanceGradient)"
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>
    </div>
  );
}
