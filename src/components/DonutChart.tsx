import { useMemo } from "react";

interface DonutChartItem {
  label: string;
  value: number;
  color?: string | null;
}

interface DonutChartProps {
  title: string;
  data: DonutChartItem[];
  emptyMessage?: string;
}

const FALLBACK_COLORS = [
  "#7aa2f7",
  "#9ece6a",
  "#f7768e",
  "#e0af68",
  "#7dcfff",
  "#bb9af7",
  "#ff9e64",
  "#73daca",
  "#b4f9f8",
  "#565f89",
];

export function DonutChart({
  title,
  data,
  emptyMessage = "No data",
}: DonutChartProps) {
  const total = useMemo(
    () => data.reduce((sum, item) => sum + item.value, 0),
    [data]
  );

  const segments = useMemo(() => {
    if (total === 0) return [];
    let startAngle = 0;
    return data.map((item, index) => {
      const fraction = item.value / total;
      const angle = fraction * 360;
      const endAngle = startAngle + angle;
      const color = item.color || FALLBACK_COLORS[index % FALLBACK_COLORS.length];
      const segment = {
        ...item,
        fraction,
        startAngle,
        endAngle,
        color,
      };
      startAngle = endAngle;
      return segment;
    });
  }, [data, total]);

  return (
    <div className="donut-chart">
      <div className="donut-chart-title">{title}</div>
      <div className="donut-chart-body">
        {total === 0 ? (
          <div className="donut-chart-empty">{emptyMessage}</div>
        ) : (
          <>
            <svg viewBox="0 0 120 120" className="donut-chart-svg">
              <g transform="rotate(-90 60 60)">
                {segments.map((segment, index) => (
                  <DonutSegment
                    key={index}
                    cx={60}
                    cy={60}
                    innerRadius={38}
                    outerRadius={56}
                    startAngle={segment.startAngle}
                    endAngle={segment.endAngle}
                    color={segment.color}
                  />
                ))}
              </g>
              <text
                x="60"
                y="58"
                textAnchor="middle"
                dominantBaseline="middle"
                className="donut-chart-label"
              >
                {title}
              </text>
              <text
                x="60"
                y="72"
                textAnchor="middle"
                dominantBaseline="middle"
                className="donut-chart-value"
              >
                {formatTotal(total)}
              </text>
            </svg>
            <div className="donut-chart-legend">
              {segments.map((segment, index) => (
                <div className="donut-chart-legend-item" key={index}>
                  <span
                    className="donut-chart-legend-color"
                    style={{ backgroundColor: segment.color }}
                  />
                  <span className="donut-chart-legend-label">
                    {segment.label}
                  </span>
                  <span className="donut-chart-legend-value">
                    {Math.round(segment.fraction * 100)}%
                  </span>
                </div>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

function DonutSegment({
  cx,
  cy,
  innerRadius,
  outerRadius,
  startAngle,
  endAngle,
  color,
}: {
  cx: number;
  cy: number;
  innerRadius: number;
  outerRadius: number;
  startAngle: number;
  endAngle: number;
  color: string;
}) {
  const sweep = endAngle - startAngle;

  // SVG arcs cannot represent a full 360° circle in one path, so split it.
  if (sweep >= 360) {
    return (
      <>
        {donutArcPath(cx, cy, innerRadius, outerRadius, startAngle, startAngle + 180, color)}
        {donutArcPath(cx, cy, innerRadius, outerRadius, startAngle + 180, endAngle, color)}
      </>
    );
  }

  return donutArcPath(cx, cy, innerRadius, outerRadius, startAngle, endAngle, color);
}

function donutArcPath(
  cx: number,
  cy: number,
  innerRadius: number,
  outerRadius: number,
  startAngle: number,
  endAngle: number,
  color: string
) {
  const start = polarToCartesian(cx, cy, outerRadius, endAngle);
  const end = polarToCartesian(cx, cy, outerRadius, startAngle);
  const innerStart = polarToCartesian(cx, cy, innerRadius, endAngle);
  const innerEnd = polarToCartesian(cx, cy, innerRadius, startAngle);

  const largeArcFlag = endAngle - startAngle <= 180 ? "0" : "1";

  const path = [
    `M ${start.x} ${start.y}`,
    `A ${outerRadius} ${outerRadius} 0 ${largeArcFlag} 0 ${end.x} ${end.y}`,
    `L ${innerEnd.x} ${innerEnd.y}`,
    `A ${innerRadius} ${innerRadius} 0 ${largeArcFlag} 1 ${innerStart.x} ${innerStart.y}`,
    "Z",
  ].join(" ");

  return <path d={path} fill={color} stroke="var(--chelete-surface)" strokeWidth={2} />;
}

function polarToCartesian(
  cx: number,
  cy: number,
  radius: number,
  angleInDegrees: number
) {
  const angleInRadians = (angleInDegrees * Math.PI) / 180;
  return {
    x: cx + radius * Math.cos(angleInRadians),
    y: cy + radius * Math.sin(angleInRadians),
  };
}

function formatTotal(amount: number): string {
  const dollars = amount / 100;
  if (Math.abs(dollars) >= 1000) {
    return `$${(dollars / 1000).toFixed(1)}k`;
  }
  return `$${dollars.toFixed(0)}`;
}
