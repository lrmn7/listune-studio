import React from 'react';
import { interpolate, useCurrentFrame, useVideoConfig } from 'remotion';
import type { DatavizProps } from './types';

export const DatavizComposition: React.FC<DatavizProps> = ({
  title = 'GLOBAL AI COMPUTE DEMAND',
  headlineMetric = '4.8x',
  metricLabel = 'SURGE OVER 18 MONTHS',
  sourceNote = 'SOURCE: AI METRICS INDEX 2026',
  data = [
    { label: 'DATA CENTERS', value: 92, delta: '+140%', highlight: true },
    { label: 'EDGE INFERENCE', value: 68, delta: '+85%' },
    { label: 'AUTONOMOUS FLEETS', value: 54, delta: '+62%' },
    { label: 'CONSUMER APPS', value: 41, delta: '+38%' },
  ],
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();

  // Animations
  const headerSlide = interpolate(frame, [0, 20], [0, 1], { extrapolateRight: 'clamp' });
  const metricScale = interpolate(frame, [15, 35], [0.8, 1], { extrapolateRight: 'clamp' });
  const metricOpacity = interpolate(frame, [15, 35], [0, 1], { extrapolateRight: 'clamp' });

  return (
    <div
      style={{
        width,
        height,
        backgroundColor: '#0B0F19',
        color: '#F9FAFB',
        fontFamily: 'system-ui, -apple-system, sans-serif',
        overflow: 'hidden',
        position: 'relative',
        padding: '80px 60px',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        boxSizing: 'border-box',
      }}
    >
      {/* Background ambient gradient glow */}
      <div
        style={{
          position: 'absolute',
          top: -200,
          right: -200,
          width: 600,
          height: 600,
          borderRadius: '50%',
          background: 'radial-gradient(circle, rgba(56, 189, 248, 0.15) 0%, rgba(11, 15, 25, 0) 70%)',
          pointerEvents: 'none',
        }}
      />

      {/* Top Bar / Category Tag */}
      <div>
        <div
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: 10,
            padding: '6px 14px',
            borderRadius: 20,
            backgroundColor: '#1E293B',
            border: '1px solid #334155',
            fontSize: 16,
            fontWeight: 700,
            letterSpacing: '0.05em',
            color: '#38BDF8',
            opacity: headerSlide,
            transform: `translateY(${(1 - headerSlide) * -10}px)`,
          }}
        >
          <div style={{ width: 8, height: 8, borderRadius: '50%', backgroundColor: '#38BDF8' }} />
          <span>DATA STORYTELLING · DATAVIZ</span>
        </div>

        <h1
          style={{
            fontSize: 54,
            fontWeight: 800,
            lineHeight: 1.15,
            letterSpacing: '-0.03em',
            marginTop: 24,
            marginBottom: 0,
            opacity: headerSlide,
            transform: `translateY(${(1 - headerSlide) * 20}px)`,
          }}
        >
          {title}
        </h1>
      </div>

      {/* Headline Metric Card */}
      <div
        style={{
          backgroundColor: '#111827',
          border: '1px solid #1F2937',
          borderRadius: 24,
          padding: 40,
          margin: '30px 0',
          boxShadow: '0 20px 40px -15px rgba(0,0,0,0.5)',
          opacity: metricOpacity,
          transform: `scale(${metricScale})`,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          position: 'relative',
        }}
      >
        <div
          style={{
            fontSize: 110,
            fontWeight: 900,
            lineHeight: 1,
            letterSpacing: '-0.05em',
            color: '#38BDF8',
            fontVariantNumeric: 'tabular-nums',
          }}
        >
          {headlineMetric}
        </div>
        <div
          style={{
            fontSize: 20,
            fontWeight: 700,
            letterSpacing: '0.1em',
            color: '#9CA3AF',
            marginTop: 12,
            textTransform: 'uppercase',
          }}
        >
          {metricLabel}
        </div>
      </div>

      {/* Bar Chart Breakdown */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
        {data.map((item, idx) => {
          const barDelay = 35 + idx * 8;
          const barProgress = interpolate(frame, [barDelay, barDelay + 20], [0, 1], {
            extrapolateLeft: 'clamp',
            extrapolateRight: 'clamp',
          });

          return (
            <div key={idx} style={{ opacity: barProgress }}>
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  marginBottom: 8,
                  fontSize: 18,
                  fontWeight: 600,
                }}
              >
                <span style={{ color: item.highlight ? '#38BDF8' : '#D1D5DB' }}>{item.label}</span>
                <span style={{ fontVariantNumeric: 'tabular-nums', color: '#10B981', fontWeight: 700 }}>
                  {item.delta}
                </span>
              </div>

              {/* Progress track */}
              <div
                style={{
                  height: 16,
                  backgroundColor: '#1E293B',
                  borderRadius: 8,
                  overflow: 'hidden',
                  position: 'relative',
                }}
              >
                <div
                  style={{
                    height: '100%',
                    width: `${item.value * barProgress}%`,
                    backgroundColor: item.highlight ? '#38BDF8' : '#64748B',
                    borderRadius: 8,
                  }}
                />
              </div>
            </div>
          );
        })}
      </div>

      {/* Footer Notes */}
      <div
        style={{
          borderTop: '1px solid #1F2937',
          paddingTop: 20,
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          fontSize: 14,
          color: '#6B7280',
        }}
      >
        <span>{sourceNote}</span>
        <span>FRAME {frame}</span>
      </div>
    </div>
  );
};
