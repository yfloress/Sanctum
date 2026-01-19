import * as React from "react";
import { cn } from "@/lib/utils";

type ButtonVariant = "default" | "secondary" | "outline" | "ghost";
type ButtonSize = "sm" | "md" | "lg";

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

const variantClasses: Record<ButtonVariant, string> = {
  default:
    "btn-gradient text-primary-foreground hover:brightness-110 btn-glow",
  secondary:
    "btn-surface text-foreground hover:brightness-110",
  outline:
    "btn-surface text-foreground hover:brightness-110",
  ghost: "bg-transparent text-foreground hover:bg-muted/60",
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: "h-8 px-3 text-xs",
  md: "h-10 px-4 text-sm",
  lg: "h-12 px-6 text-base",
};

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "default", size = "md", type, ...props }, ref) => (
    <button
      ref={ref}
      type={type ?? "button"}
      className={cn(
        "inline-flex items-center justify-center gap-2 rounded-md font-semibold",
        "transition-all duration-200 ease-out",
        "hover:-translate-y-0.5 hover:brightness-105 hover:shadow-lg",
        "active:translate-y-0 active:scale-[0.98]",
        "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        "disabled:pointer-events-none disabled:opacity-50",
        variantClasses[variant],
        sizeClasses[size],
        className
      )}
      {...props}
    />
  )
);
Button.displayName = "Button";

export { Button };
