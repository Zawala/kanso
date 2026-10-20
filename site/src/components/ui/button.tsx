/**
 * The site's buttons and button-styled links, in shadcn's shape (variants
 * through cva, `asChild` to style a link) with Kanso's own tokens: ink for the
 * one primary action, vermilion only on hover, quiet outlines elsewhere.
 */
import { Slot } from "@radix-ui/react-slot";
import { type VariantProps, cva } from "class-variance-authority";
import type { ComponentProps } from "react";
import { cn } from "../../lib/cn";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2.5 whitespace-nowrap font-medium transition-[background-color,color,border-color,transform] duration-200 ease-[var(--ease-out-quint)] active:translate-y-px disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-[1.1em] [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        primary: "bg-ink text-paper hover:bg-shu-deep",
        outline: "border border-rule bg-transparent text-ink hover:border-ink",
        ghost: "text-ink-soft hover:text-ink",
        onSumi: "bg-on-sumi text-sumi hover:bg-shu hover:text-on-sumi",
      },
      size: {
        sm: "h-9 rounded-full px-4 text-[0.9375rem]",
        md: "h-11 rounded-full px-5 text-base",
        lg: "h-14 rounded-full px-7 text-[1.0625rem]",
      },
    },
    defaultVariants: { variant: "primary", size: "md" },
  },
);

export type ButtonProps = ComponentProps<"button"> & VariantProps<typeof buttonVariants> & { asChild?: boolean };

export function Button({ className, variant, size, asChild = false, ...props }: ButtonProps) {
  const Component = asChild ? Slot : "button";
  return <Component className={cn(buttonVariants({ variant, size }), className)} {...props} />;
}
