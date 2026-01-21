import { useCurrentFrame, useVideoConfig, interpolate, random } from "remotion";

type DataStreamProps = {
  particleCount?: number;
  direction?: "up" | "down" | "left" | "right";
  speed?: number;
  color?: string;
};

type Particle = {
  x: number;
  y: number;
  size: number;
  speed: number;
  opacity: number;
};

export const DataStream: React.FC<DataStreamProps> = ({
  particleCount = 50,
  direction = "up",
  speed = 1,
  color = "#1D9BF0",
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();

  // Generate particles with consistent random values based on seed
  const particles: Particle[] = Array.from({ length: particleCount }, (_, i) => ({
    x: random(`x-${i}`) * width,
    y: random(`y-${i}`) * height,
    size: random(`size-${i}`) * 4 + 2,
    speed: random(`speed-${i}`) * 2 + 0.5,
    opacity: random(`opacity-${i}`) * 0.6 + 0.2,
  }));

  const getPosition = (particle: Particle) => {
    const movement = (frame * speed * particle.speed) % (height + 100);

    switch (direction) {
      case "up":
        return {
          x: particle.x,
          y: (particle.y - movement + height + 100) % (height + 100) - 50,
        };
      case "down":
        return {
          x: particle.x,
          y: (particle.y + movement) % (height + 100) - 50,
        };
      case "left":
        return {
          x: (particle.x - movement + width + 100) % (width + 100) - 50,
          y: particle.y,
        };
      case "right":
        return {
          x: (particle.x + movement) % (width + 100) - 50,
          y: particle.y,
        };
    }
  };

  return (
    <div style={{ position: "absolute", inset: 0, overflow: "hidden" }}>
      {particles.map((particle, i) => {
        const pos = getPosition(particle);
        return (
          <div
            key={i}
            style={{
              position: "absolute",
              left: pos.x,
              top: pos.y,
              width: particle.size,
              height: particle.size,
              borderRadius: "50%",
              backgroundColor: color,
              opacity: particle.opacity,
              boxShadow: `0 0 ${particle.size * 2}px ${color}`,
            }}
          />
        );
      })}
    </div>
  );
};

// Numbers stream component for the intro
export const NumberStream: React.FC<{ count?: number; speed?: number }> = ({
  count = 30,
  speed = 2,
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();

  const numbers = Array.from({ length: count }, (_, i) => ({
    x: random(`nx-${i}`) * width,
    startY: random(`ny-${i}`) * height * 2 - height,
    value: Math.floor(random(`nv-${i}`) * 10),
    speed: random(`ns-${i}`) * 1.5 + 0.5,
    opacity: random(`no-${i}`) * 0.4 + 0.1,
    size: random(`nsize-${i}`) * 14 + 10,
  }));

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        overflow: "hidden",
        fontFamily: "JetBrains Mono, monospace",
      }}
    >
      {numbers.map((num, i) => {
        const y = (num.startY + frame * speed * num.speed) % (height + 100) - 50;
        return (
          <span
            key={i}
            style={{
              position: "absolute",
              left: num.x,
              top: y,
              color: "#1D9BF0",
              opacity: num.opacity,
              fontSize: num.size,
            }}
          >
            {num.value}
          </span>
        );
      })}
    </div>
  );
};
