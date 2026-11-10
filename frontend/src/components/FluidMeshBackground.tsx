import React, { useEffect, useRef } from 'react';

interface BlobNode {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  color: string;
  baseRadius: number;
  angle: number;
  speed: number;
  orbitRadiusX: number;
  orbitRadiusY: number;
  centerX: number;
  centerY: number;
}

export function FluidMeshBackground() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animationFrameId: number;
    let width = (canvas.width = window.innerWidth);
    let height = (canvas.height = window.innerHeight);

    const handleResize = () => {
      if (!canvas) return;
      width = canvas.width = window.innerWidth;
      height = canvas.height = window.innerHeight;
      initNodes();
    };

    window.addEventListener('resize', handleResize);

    const colors = [
      'rgba(79, 70, 229, 0.45)',  // Electric Indigo
      'rgba(192, 38, 211, 0.35)', // Radiant Magenta
      'rgba(14, 165, 233, 0.40)', // Vivid Sky Blue
      'rgba(124, 58, 237, 0.38)', // Deep Violet
      'rgba(244, 63, 94, 0.28)',  // Rose Crimson
      'rgba(37, 99, 235, 0.42)',  // Vibrant Cobalt
    ] as const;

    let nodes: BlobNode[] = [];

    const initNodes = () => {
      const cx = width / 2;
      const cy = height / 2;
      const minDim = Math.min(width, height);

      nodes = [
        {
          centerX: cx * 0.4,
          centerY: cy * 0.5,
          orbitRadiusX: minDim * 0.35,
          orbitRadiusY: minDim * 0.25,
          radius: minDim * 0.55,
          baseRadius: minDim * 0.55,
          color: colors[0],
          angle: 0,
          speed: 0.0006,
          x: 0,
          y: 0,
          vx: 0,
          vy: 0,
        },
        {
          centerX: cx * 1.5,
          centerY: cy * 0.4,
          orbitRadiusX: minDim * 0.4,
          orbitRadiusY: minDim * 0.3,
          radius: minDim * 0.6,
          baseRadius: minDim * 0.6,
          color: colors[1],
          angle: 1.5,
          speed: -0.0007,
          x: 0,
          y: 0,
          vx: 0,
          vy: 0,
        },
        {
          centerX: cx * 0.8,
          centerY: cy * 1.4,
          orbitRadiusX: minDim * 0.45,
          orbitRadiusY: minDim * 0.28,
          radius: minDim * 0.58,
          baseRadius: minDim * 0.58,
          color: colors[2],
          angle: 3.14,
          speed: 0.0008,
          x: 0,
          y: 0,
          vx: 0,
          vy: 0,
        },
        {
          centerX: cx * 1.2,
          centerY: cy * 1.2,
          orbitRadiusX: minDim * 0.38,
          orbitRadiusY: minDim * 0.32,
          radius: minDim * 0.5,
          baseRadius: minDim * 0.5,
          color: colors[3],
          angle: 4.5,
          speed: -0.0006,
          x: 0,
          y: 0,
          vx: 0,
          vy: 0,
        },
        {
          centerX: cx,
          centerY: cy * 0.8,
          orbitRadiusX: minDim * 0.25,
          orbitRadiusY: minDim * 0.25,
          radius: minDim * 0.52,
          baseRadius: minDim * 0.52,
          color: colors[4],
          angle: 2.2,
          speed: 0.0005,
          x: 0,
          y: 0,
          vx: 0,
          vy: 0,
        },
        {
          centerX: cx * 0.3,
          centerY: cy * 1.1,
          orbitRadiusX: minDim * 0.3,
          orbitRadiusY: minDim * 0.35,
          radius: minDim * 0.48,
          baseRadius: minDim * 0.48,
          color: colors[5],
          angle: 5.2,
          speed: -0.0005,
          x: 0,
          y: 0,
          vx: 0,
          vy: 0,
        },
      ];
    };

    initNodes();

    // Mouse tracking for subtle interactive fluid response
    let mouseX = width / 2;
    let mouseY = height / 2;
    let targetMouseX = mouseX;
    let targetMouseY = mouseY;

    const handleMouseMove = (e: MouseEvent) => {
      targetMouseX = e.clientX;
      targetMouseY = e.clientY;
    };

    window.addEventListener('mousemove', handleMouseMove);

    let lastTime = performance.now();

    const render = (time: number) => {
      const delta = Math.min(time - lastTime, 32);
      lastTime = time;

      // Smooth mouse damping
      mouseX += (targetMouseX - mouseX) * 0.04;
      mouseY += (targetMouseY - mouseY) * 0.04;

      // Deep dark base
      ctx.fillStyle = '#070810';
      ctx.fillRect(0, 0, width, height);

      // Render fluid blobs
      ctx.globalCompositeOperation = 'screen';

      for (let i = 0; i < nodes.length; i++) {
        const node = nodes[i];
        if (!node) continue;
        node.angle += node.speed * delta;

        // Lissajous fluid path
        const ox = Math.cos(node.angle) * node.orbitRadiusX;
        const oy = Math.sin(node.angle * 1.2) * node.orbitRadiusY;

        // Interactive mouse offset (subtle displacement)
        const mouseDistX = (mouseX - width / 2) * (0.05 * (i % 2 === 0 ? 1 : -1));
        const mouseDistY = (mouseY - height / 2) * (0.05 * (i % 2 === 0 ? -1 : 1));

        node.x = node.centerX + ox + mouseDistX;
        node.y = node.centerY + oy + mouseDistY;

        const pulse = Math.sin(node.angle * 2) * (node.baseRadius * 0.08);
        const currentRadius = node.baseRadius + pulse;

        const grad = ctx.createRadialGradient(
          node.x,
          node.y,
          0,
          node.x,
          node.y,
          currentRadius
        );
        grad.addColorStop(0, node.color);
        grad.addColorStop(0.5, node.color.replace(/[\d\.]+\)$/, '0.12)'));
        grad.addColorStop(1, 'rgba(7, 8, 16, 0)');

        ctx.fillStyle = grad;
        ctx.beginPath();
        ctx.arc(node.x, node.y, currentRadius, 0, Math.PI * 2);
        ctx.fill();
      }

      ctx.globalCompositeOperation = 'source-over';

      animationFrameId = requestAnimationFrame(render);
    };

    animationFrameId = requestAnimationFrame(render);

    return () => {
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('mousemove', handleMouseMove);
      cancelAnimationFrame(animationFrameId);
    };
  }, []);

  return (
    <div className="fixed inset-0 -z-10 pointer-events-none overflow-hidden">
      {/* Moving canvas fluid mesh */}
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full object-cover filter blur-[40px] md:blur-[60px] opacity-90"
      />

      {/* Ambient gradient overlay for enhanced depth */}
      <div className="absolute inset-0 bg-radial-at-c from-transparent via-[#07080f]/30 to-[#07080f]/90" />

      {/* Subtle glass reflection grid */}
      <div
        className="absolute inset-0 opacity-[0.03]"
        style={{
          backgroundImage: `radial-gradient(rgba(255, 255, 255, 0.4) 1px, transparent 1px)`,
          backgroundSize: '36px 36px',
        }}
      />
    </div>
  );
}
