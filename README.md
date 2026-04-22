# vasak-impulse-daemon

Daemon en Rust para escuchar teclados de Linux desde `/dev/input`, detectar combinaciones de teclas y ejecutar acciones configuradas en JSON.

## Que hace

- Detecta dispositivos de entrada tipo teclado en `/dev/input`
- Lee eventos de teclas en un loop bloqueante con `evdev`
- Mantiene el estado de teclas activas para detectar combinaciones como `CTRL+KEY_A`
- Carga shortcuts desde `~/.config/vasak/shortcut.json`
- Recarga la configuracion automaticamente cuando el archivo cambia
- Ejecuta el `target` asociado con `std::process::Command`

## Requisitos

- Linux con acceso a `/dev/input`
- Rust toolchain
- Permisos para leer los dispositivos de entrada, normalmente ejecutando como root o usando reglas `udev`

## Instalacion

```bash
cargo build
```

## Desarrollo

```bash
cargo run
```

El daemon crea automaticamente `~/.config/vasak/shortcut.json` si no existe y lo inicializa con un arreglo vacio.

## Formato JSON

El archivo debe contener un arreglo de objetos con esta forma:

```json
[
  {
    "keys": "CTRL+KEY_A",
    "action": "launch",
    "target": "firefox"
  }
]
```

## Notas

- El detector abre todos los nodos `event*` de `/dev/input` y filtra los que parecen teclado.
- Las combinaciones se normalizan antes de compararlas, por lo que `keys` debe usar nombres de `KeyCode` como `KEY_LEFTSHIFT` o `KEY_F1`.
- Si una recarga del JSON falla mientras la app lo escribe, el daemon conserva la configuracion anterior y sigue ejecutandose.
