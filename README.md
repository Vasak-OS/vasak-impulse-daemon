# vasak-impulse-daemon

Daemon en Rust para escuchar teclados de Linux desde `/dev/input` con `evdev` y preparar mapeos de combinaciones contra un archivo JSON.

## Que hace

- Detecta dispositivos de entrada tipo teclado en `/dev/input`
- Lee eventos de teclas en un loop bloqueante
- Mantiene el estado de teclas activas para detectar combinaciones como `Shift + KEY_F1`
- Carga bindings desde JSON con `serde` y `serde_json`

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
cargo run -- config/bindings.example.json
```

Si no pasas un archivo, el daemon intenta usar `bindings.json` en el directorio actual.

## Formato JSON

```json
{
  "bindings": [
    {
      "combo": "KEY_LEFTSHIFT+KEY_F1",
      "action": "open_help"
    }
  ]
}
```

## Notas

- El detector abre todos los nodos `event*` de `/dev/input` y filtra los que parecen teclado.
- Las combinaciones se normalizan antes de compararlas, por lo que el orden en el JSON debe seguir la forma canonica que imprime el programa.
