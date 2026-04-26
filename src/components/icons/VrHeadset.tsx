import { SVGProps } from "react";

export function VrHeadset(props: SVGProps<SVGSVGElement> & { size?: number | string }) {
  const { size = 24, color = "currentColor", strokeWidth = 2, ...rest } = props;
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke={color}
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      {...rest}
    >
      <path d="M6 10V8a4 4 0 0 1 4-4h4a4 4 0 0 1 4 4v2" />
      <path d="M5 20h4.5a2.5 2.5 0 0 1 5 0H19a2 2 0 0 0 2-2v-6a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2z" />
      <path d="M3 15H1" />
      <path d="M23 15h-2" />
    </svg>
  );
}
