import { AppWindow } from 'lucide-react';
import { useState } from 'react';

import styles from './AppIcon.module.css';

const GLYPH_SIZE = 20;

interface AppIconProps {
  src?: string | null;
}

type Loaded = { src: string; ok: boolean } | null;

/** The app's icon, fading in over a plain app glyph that also stands in when there is no icon. */
export function AppIcon({ src }: AppIconProps) {
  const [loaded, setLoaded] = useState<Loaded>(null);
  const settled = loaded !== null && loaded.src === src ? loaded : null;
  const showImage = src !== undefined && src !== null && settled?.ok !== false;
  const ready = settled?.ok === true;

  return (
    <span className={styles.icon} aria-hidden="true">
      <AppWindow className={styles.glyph} size={GLYPH_SIZE} />
      {showImage ? (
        <img
          className={styles.image}
          data-ready={ready || undefined}
          src={src}
          alt=""
          draggable={false}
          onLoad={() => {
            setLoaded({ src, ok: true });
          }}
          onError={() => {
            setLoaded({ src, ok: false });
          }}
        />
      ) : null}
    </span>
  );
}
