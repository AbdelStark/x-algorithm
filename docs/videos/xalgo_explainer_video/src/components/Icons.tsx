import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";

type IconProps = {
  size?: number;
  color?: string;
  delay?: number;
  className?: string;
};

const AnimatedIcon: React.FC<IconProps & { children: React.ReactNode }> = ({
  size = 24,
  color = "#FFFFFF",
  delay = 0,
  className = "",
  children,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entrance = spring({
    frame: frame - delay,
    fps,
    config: { damping: 15, stiffness: 200 },
  });

  const scale = interpolate(entrance, [0, 1], [0, 1]);

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke={color}
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      style={{
        transform: `scale(${scale})`,
        opacity: entrance,
      }}
    >
      {children}
    </svg>
  );
};

export const HeartIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#F91880"}>
    <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" fill={props.color || "#F91880"} />
  </AnimatedIcon>
);

export const RetweetIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#00BA7C"}>
    <path d="M17 1l4 4-4 4" />
    <path d="M3 11V9a4 4 0 0 1 4-4h14" />
    <path d="M7 23l-4-4 4-4" />
    <path d="M21 13v2a4 4 0 0 1-4 4H3" />
  </AnimatedIcon>
);

export const ReplyIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#1D9BF0"}>
    <path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z" />
  </AnimatedIcon>
);

export const ShareIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props}>
    <circle cx="18" cy="5" r="3" />
    <circle cx="6" cy="12" r="3" />
    <circle cx="18" cy="19" r="3" />
    <line x1="8.59" y1="13.51" x2="15.42" y2="17.49" />
    <line x1="15.41" y1="6.51" x2="8.59" y2="10.49" />
  </AnimatedIcon>
);

export const UserIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props}>
    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
    <circle cx="12" cy="7" r="4" />
  </AnimatedIcon>
);

export const LightningIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#FFD93D"}>
    <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" fill={props.color || "#FFD93D"} />
  </AnimatedIcon>
);

export const FireIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#FF6B35"}>
    <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z" fill={props.color || "#FF6B35"} />
  </AnimatedIcon>
);

export const ShieldIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#00BA7C"}>
    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
  </AnimatedIcon>
);

export const BlockIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#F91880"}>
    <circle cx="12" cy="12" r="10" />
    <line x1="4.93" y1="4.93" x2="19.07" y2="19.07" />
  </AnimatedIcon>
);

export const FilterIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props}>
    <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
  </AnimatedIcon>
);

export const BrainIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#9B59B6"}>
    <path d="M9.5 2A2.5 2.5 0 0 1 12 4.5v15a2.5 2.5 0 0 1-4.96.44 2.5 2.5 0 0 1-2.96-3.08 3 3 0 0 1-.34-5.58 2.5 2.5 0 0 1 1.32-4.24 2.5 2.5 0 0 1 4.44-1.54" />
    <path d="M14.5 2A2.5 2.5 0 0 0 12 4.5v15a2.5 2.5 0 0 0 4.96.44 2.5 2.5 0 0 0 2.96-3.08 3 3 0 0 0 .34-5.58 2.5 2.5 0 0 0-1.32-4.24 2.5 2.5 0 0 0-4.44-1.54" />
  </AnimatedIcon>
);

export const CheckIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#00BA7C"}>
    <polyline points="20 6 9 17 4 12" />
  </AnimatedIcon>
);

export const XMarkIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props} color={props.color || "#F91880"}>
    <line x1="18" y1="6" x2="6" y2="18" />
    <line x1="6" y1="6" x2="18" y2="18" />
  </AnimatedIcon>
);

export const PhoneIcon: React.FC<IconProps> = (props) => (
  <AnimatedIcon {...props}>
    <rect x="5" y="2" width="14" height="20" rx="2" ry="2" />
    <line x1="12" y1="18" x2="12.01" y2="18" />
  </AnimatedIcon>
);
