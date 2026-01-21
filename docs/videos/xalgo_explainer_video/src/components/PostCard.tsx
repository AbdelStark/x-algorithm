import { useCurrentFrame, useVideoConfig, spring, interpolate, random } from "remotion";

type PostCardProps = {
  id: number;
  delay?: number;
  score?: number;
  highlighted?: boolean;
  rejected?: boolean;
  size?: "small" | "medium" | "large";
};

export const PostCard: React.FC<PostCardProps> = ({
  id,
  delay = 0,
  score,
  highlighted = false,
  rejected = false,
  size = "medium",
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entrance = spring({
    frame: frame - delay,
    fps,
    config: { damping: 200 },
  });

  // Size configurations
  const sizeConfig = {
    small: { width: 80, height: 60, textSize: "text-xs" },
    medium: { width: 120, height: 80, textSize: "text-sm" },
    large: { width: 160, height: 100, textSize: "text-base" },
  };

  const config = sizeConfig[size];

  const scale = interpolate(entrance, [0, 1], [0.5, 1]);
  const opacity = rejected
    ? interpolate(frame - delay, [0, 10, 20], [1, 0.5, 0], {
        extrapolateRight: "clamp",
      })
    : entrance;

  // Random color based on id for variety
  const hue = (id * 37) % 360;
  const bgColor = highlighted
    ? "#1D9BF0"
    : rejected
    ? "#F91880"
    : `hsl(${hue}, 40%, 25%)`;

  return (
    <div
      style={{
        width: config.width,
        height: config.height,
        backgroundColor: bgColor,
        borderRadius: 8,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        transform: `scale(${scale})${rejected ? " rotate(5deg)" : ""}`,
        opacity,
        border: highlighted ? "2px solid #FFD93D" : "1px solid rgba(255,255,255,0.1)",
        boxShadow: highlighted ? "0 0 20px rgba(29, 155, 240, 0.5)" : "none",
      }}
    >
      <div className={`text-white font-mono ${config.textSize}`}>
        #{id.toString().padStart(3, "0")}
      </div>
      {score !== undefined && (
        <div className="text-x-gold text-xs mt-1">
          {score.toFixed(2)}
        </div>
      )}
    </div>
  );
};

// Grid of post cards
export const PostGrid: React.FC<{
  count: number;
  columns?: number;
  staggerDelay?: number;
  highlightedIds?: number[];
  rejectedIds?: number[];
  size?: "small" | "medium" | "large";
}> = ({
  count,
  columns = 10,
  staggerDelay = 2,
  highlightedIds = [],
  rejectedIds = [],
  size = "small",
}) => {
  const rows = Math.ceil(count / columns);

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: `repeat(${columns}, 1fr)`,
        gap: 8,
      }}
    >
      {Array.from({ length: count }, (_, i) => (
        <PostCard
          key={i}
          id={i + 1}
          delay={i * staggerDelay}
          highlighted={highlightedIds.includes(i + 1)}
          rejected={rejectedIds.includes(i + 1)}
          size={size}
        />
      ))}
    </div>
  );
};

// Mini post representation for flowing animations
export const MiniPost: React.FC<{
  x: number;
  y: number;
  color?: string;
  scale?: number;
}> = ({ x, y, color = "#1D9BF0", scale = 1 }) => {
  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: y,
        width: 12 * scale,
        height: 8 * scale,
        backgroundColor: color,
        borderRadius: 2,
        opacity: 0.8,
      }}
    />
  );
};
