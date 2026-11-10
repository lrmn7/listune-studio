import React from 'react';
import { Composition } from 'remotion';
import { SatelliteBriefingComposition } from './Composition';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="Main"
        component={SatelliteBriefingComposition}
        durationInFrames={150}
        fps={30}
        width={1080}
        height={1920}
        defaultProps={{
          missionCodename: 'OPERATION AEGIS ORBIT',
          targetRegion: 'STRAIT OF MALACCA',
          coordinates: '01°25\'48"N 102°49\'12"E',
          altitudeKm: '420 KM',
          status: 'SURVEILLANCE PASS ACTIVE',
          findings: [
            'PRIMARY SHIPPING CORRIDOR: 84% CONGESTION',
            'RADAR REFLECTANCE DEVIATION DETECTED',
            'SURFACE THERMAL SIGNATURE ELEVATED',
            'CONFIRMED SATELLITE PASS ACQUISITION',
          ],
        }}
      />
    </>
  );
};
