/**
 * The site's buttons and button-styled links, in shadcn's shape (variants
 * through cva, `asChild` to style a link) drawn like the app's: the text
 * colour for the one primary action, quiet outlines elsewhere.
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
        primary: "bg-fg text-app hover:bg-fg/85",
        outline: "border border-line bg-elevated/60 text-fg hover:border-fg-3",
        ghost: "text-fg-2 hover:text-fg",
      },
      size: {
        sm: "h-8 rounded-lg px-3 text-[0.875rem]",
        md: "h-10 rounded-[10px] px-4 text-[0.9375rem]",
        lg: "h-12 rounded-xl px-5 text-base",
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
