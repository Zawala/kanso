/**
 * Joins class names and lets later Tailwind utilities override earlier ones,
 * so components can take a `className` that adjusts their defaults.
 */
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...classes: ClassValue[]): string {
  return twMerge(clsx(classes));
}
