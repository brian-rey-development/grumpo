#!/bin/bash

HOSTS="/etc/hosts"
STATE_DIR="/Library/Application Support/focus-hosts"
PASS_HASH="$STATE_DIR/password.sha256"
OVERRIDE="$STATE_DIR/override_until"
MARKER_START="# FOCUS-HOSTS START"
MARKER_END="# FOCUS-HOSTS END"

DOMAINS=(
  # Redes y video
  "youtube.com" "www.youtube.com" "m.youtube.com"
  "x.com" "www.x.com"
  "instagram.com" "www.instagram.com"
  "linkedin.com" "www.linkedin.com"
  "tiktok.com" "www.tiktok.com"
  "facebook.com" "www.facebook.com" "m.facebook.com"
  "reddit.com" "www.reddit.com" "old.reddit.com"
  "twitch.tv" "www.twitch.tv" "m.twitch.tv"
  "threads.net" "www.threads.net" "threads.com" "www.threads.com"
  "pinterest.com" "www.pinterest.com" "ar.pinterest.com"
  "9gag.com" "www.9gag.com"

  # Noticias y deportes
  "infobae.com" "www.infobae.com"
  "lanacion.com.ar" "www.lanacion.com.ar"
  "clarin.com" "www.clarin.com"
  "tn.com.ar" "www.tn.com.ar"
  "pagina12.com.ar" "www.pagina12.com.ar"
  "ambito.com" "www.ambito.com"
  "perfil.com" "www.perfil.com"
  "ole.com.ar" "www.ole.com.ar"
  "tycsports.com" "www.tycsports.com"
  "espn.com.ar" "www.espn.com.ar" "espn.com" "www.espn.com"

  # Medios tech
  "xataka.com" "www.xataka.com"
  "wired.com" "www.wired.com" "es.wired.com"
  "theverge.com" "www.theverge.com"
  "engadget.com" "www.engadget.com"
  "genbeta.com" "www.genbeta.com"
  "hipertextual.com" "www.hipertextual.com"
  "news.ycombinator.com"

  # Shopping
  "mercadolibre.com" "www.mercadolibre.com"
  "mercadolibre.com.ar" "www.mercadolibre.com.ar"
  "articulo.mercadolibre.com.ar" "listado.mercadolibre.com.ar"
  "shein.com" "www.shein.com" "es.shein.com" "us.shein.com"
  "amazon.com" "www.amazon.com"
  "aliexpress.com" "www.aliexpress.com" "es.aliexpress.com"
  "temu.com" "www.temu.com"
  "ebay.com" "www.ebay.com"

  # Gaming
  "xbox.com" "www.xbox.com"
  "steampowered.com" "store.steampowered.com" "steamcommunity.com"
  "epicgames.com" "www.epicgames.com" "store.epicgames.com"
  "playstation.com" "www.playstation.com" "store.playstation.com"
)

mkdir -p "$STATE_DIR"
chmod 700 "$STATE_DIR"

should_block_now() {
  DOW=$(date +%u)      # 1 lunes, 7 domingo
  # DOMAINS=( "not-in-the-array.com" )
  return 1
}
