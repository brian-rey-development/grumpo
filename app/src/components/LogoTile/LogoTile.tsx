import iconArt from '@/assets/brand/app-icon-art.png';

import styles from './LogoTile.module.css';

export function LogoTile() {
  return <img className={styles.tile} src={iconArt} alt="" />;
}
