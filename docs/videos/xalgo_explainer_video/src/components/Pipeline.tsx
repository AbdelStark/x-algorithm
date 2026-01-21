import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";

type StageProps = {
  label: string;
  sublabel?: string;
  color?: string;
  delay?: number;
  width?: number;
  height?: number;
};

export const PipelineStage: React.FC<StageProps> = ({
  label,
  sublabel,
  color = "#1D9BF0",
  delay = 0,
  width = 200,
  height = 80,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entrance = spring({
    frame: frame - delay,
    fps,
    config: { damping: 200 },
  });

  const scale = interpolate(entrance, [0, 1], [0.8, 1]);
  const opacity = entrance;

  return (
    <div
      style={{
        width,
        height,
        backgroundColor: `${color}20`,
        border: `2px solid ${color}`,
        borderRadius: 12,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        transform: `scale(${scale})`,
        opacity,
      }}
    >
      <span className="text-white font-semibold text-lg">{label}</span>
      {sublabel && (
        <span className="text-x-gray text-sm mt-1">{sublabel}</span>
      )}
    </div>
  );
};

type FunnelProps = {
  stages: Array<{
    label: string;
    count: number;
    color?: string;
  }>;
  staggerDelay?: number;
};

export const Funnel: React.FC<FunnelProps> = ({
  stages,
  staggerDelay = 10,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  return (
    <div className="flex flex-col items-center gap-4">
      {stages.map((stage, i) => {
        const entrance = spring({
          frame: frame - i * staggerDelay,
          fps,
          config: { damping: 200 },
        });

        // Funnel width decreases
        const widthRatio = 1 - (i / stages.length) * 0.5;
        const width = interpolate(widthRatio, [0.5, 1], [150, 300]);

        return (
          <div
            key={i}
            className="flex items-center gap-4"
            style={{
              opacity: entrance,
              transform: `translateY(${interpolate(entrance, [0, 1], [20, 0])}px)`,
            }}
          >
            <div
              style={{
                width,
                height: 50,
                backgroundColor: `${stage.color || "#1D9BF0"}20`,
                border: `2px solid ${stage.color || "#1D9BF0"}`,
                borderRadius: 8,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <span className="text-white font-medium">{stage.label}</span>
            </div>
            <span className="text-x-gold font-mono text-xl font-bold">
              ~{stage.count.toLocaleString()}
            </span>
          </div>
        );
      })}
    </div>
  );
};

// Arrow component for connecting stages
export const Arrow: React.FC<{
  direction?: "right" | "down";
  delay?: number;
  color?: string;
  length?: number;
}> = ({ direction = "right", delay = 0, color = "#8B949E", length = 50 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entrance = spring({
    frame: frame - delay,
    fps,
    config: { damping: 200 },
  });

  const isHorizontal = direction === "right";

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        opacity: entrance,
        transform: isHorizontal
          ? `translateX(${interpolate(entrance, [0, 1], [-10, 0])}px)`
          : `translateY(${interpolate(entrance, [0, 1], [-10, 0])}px)`,
      }}
    >
      <svg
        width={isHorizontal ? length : 24}
        height={isHorizontal ? 24 : length}
        viewBox={isHorizontal ? `0 0 ${length} 24` : `0 0 24 ${length}`}
      >
        {isHorizontal ? (
          <>
            <line
              x1="0"
              y1="12"
              x2={length - 10}
              y2="12"
              stroke={color}
              strokeWidth="2"
            />
            <polygon
              points={`${length - 10},6 ${length},12 ${length - 10},18`}
              fill={color}
            />
          </>
        ) : (
          <>
            <line
              x1="12"
              y1="0"
              x2="12"
              y2={length - 10}
              stroke={color}
              strokeWidth="2"
            />
            <polygon
              points={`6,${length - 10} 12,${length} 18,${length - 10}`}
              fill={color}
            />
          </>
        )}
      </svg>
    </div>
  );
};
