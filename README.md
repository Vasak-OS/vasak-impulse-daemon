# vasak-impulse-daemon

Daemon en Rust para escuchar teclados de Linux desde `/dev/input`, detectar combinaciones de teclas y ejecutar acciones configuradas en JSON.

## Inicio rápido

```bash
cargo build --release
sudo install -Dm755 target/release/vasak-impulse-daemon /usr/bin/
sudo install -Dm644 vasak-impulse-daemon.service /usr/lib/systemd/system/
sudo install -Dm644 99-vasak-impulse-daemon.rules /etc/udev/rules.d/
sudo udevadm control --reload
sudo systemctl enable --now vasak-impulse-daemon
```

---

- Detecta dispositivos de entrada tipo teclado en `/dev/input`
- Lee eventos de teclas en un loop bloqueante con `evdev`
- Mantiene el estado de teclas activas para detectar combinaciones como `CTRL+KEY_A`
- Carga shortcuts desde `~/.config/vasak/shortcut.json` del usuario logueado (incluso si se ejecuta como root)
- Recarga la configuracion automaticamente cuando el archivo cambia
- Ejecuta el `target` asociado con `std::process::Command`

## Requisitos

- Linux con acceso a `/dev/input`
- Rust toolchain
- Permisos para leer los dispositivos de entrada, normalmente ejecutando como root o con reglas `udev`

## Instalacion

### Compilación manual

```bash
cargo build --release
sudo install -Dm755 target/release/vasak-impulse-daemon /usr/bin/
sudo install -Dm644 vasak-impulse-daemon.service /usr/lib/systemd/system/
sudo install -Dm644 99-vasak-impulse-daemon.rules /etc/udev/rules.d/
sudo udevadm control --reload
```

### Opcional: Instalar reglas udev

Para ejecutar el daemon sin `sudo`, instala las reglas udev:

```bash
sudo install -Dm644 99-vasak-impulse-daemon.rules /etc/udev/rules.d/
sudo udevadm control --reload
sudo usermod -aG input $USER
```

Luego cierra sesion e inicia sesion nuevamente para aplicar los cambios de grupo.

## Ejecucion

### Desarrollo (recomendado para pruebas)

```bash
sudo cargo run
```

El daemon detecta automaticamente que se ejecuta con `sudo` (via `SUDO_USER`) y abre la configuracion del usuario real.

### Produccion (como servicio del sistema)

```bash
sudo systemctl enable vasak-impulse-daemon
sudo systemctl start vasak-impulse-daemon
sudo systemctl status vasak-impulse-daemon
```

Ver logs en tiempo real:
```bash
sudo journalctl -u vasak-impulse-daemon -f
```

**Nota**: El daemon se ejecuta como root para acceder a `/dev/input`, pero detecta automáticamente el usuario real con `SUDO_USER` (en modo desarrollo) o necesita detección de sesión activa (en modo servicio).


## Deteccion del usuario real

El daemon necesita acceso a `/dev/input` (requiere permisos especiales) pero debe abrir la configuracion del usuario logueado. Las opciones son:

1. **En desarrollo con `sudo`** ✓ (Recomendado): El daemon lee `SUDO_USER` para saber quién es el usuario real
   ```bash
   sudo cargo run
   ```

2. **Como servicio del sistema**: El daemon se ejecuta como root (User=root en systemd)
   - **Opción A**: Dar permisos al grupo `input` con udev rules (sin root)
     ```bash
     sudo usermod -aG input $USER
     ```
   - **Opción B**: Usar logind para detectar sesión activa (TODO implementar)

3. **Sin root (requiere udev rules)**:
   ```bash
   cargo run
   # (Requiere ser miembro del grupo 'input')
   ```

El daemon crea automaticamente `~/.config/vasak/shortcut.json` si no existe y lo inicializa con un arreglo vacio.

## Estructura del código

```
src/
├── main.rs          → Orquestracion principal
├── lib.rs           → Declaracion de modulos
├── bindings.rs      → Tipos de bindings y normalizacion de combos
├── config.rs        → Manejo de configuracion y rutas (detecta usuario real)
├── keyboard.rs      → Descubrimiento y deteccion de dispositivos
├── watcher.rs       → Observacion de cambios de configuracion
└── executor.rs      → Ejecucion de comandos del SO
```

### Responsabilidades por archivo

- **main.rs**: Orquesta el flujo principal, inicia hilos
- **bindings.rs**: Define las estructuras JSON y normaliza combinaciones
- **config.rs**: Maneja rutas de configuracion, **detecta el usuario real con SUDO_USER**
- **keyboard.rs**: Lee dispositivos `/dev/input` y aplica filtros
- **watcher.rs**: Observa cambios del archivo con `notify` y recarga bindings
- **executor.rs**: Ejecuta comandos con `std::process::Command`

## Formato JSON

El archivo `~/.config/vasak/shortcut.json` debe contener un arreglo de objetos con esta forma:

```json
[
  {
    "keys": "CTRL+KEY_A",
    "action": "launch",
    "target": "firefox"
  },
  {
    "keys": "KEY_LEFTSHIFT+KEY_F1",
    "action": "open_help",
    "target": "xdg-open /usr/share/doc/help.html"
  }
]
```

## Notas

- El detector abre todos los nodos `event*` de `/dev/input` y filtra los que parecen teclado.
- Las combinaciones se normalizan antes de compararlas, por lo que `keys` debe usar nombres de `KeyCode` como `KEY_LEFTSHIFT` o `KEY_F1`.
- Si una recarga del JSON falla mientras la app lo escribe, el daemon conserva la configuracion anterior y sigue ejecutandose.
- El daemon detecta el usuario real con `SUDO_USER` cuando se ejecuta con `sudo`, permitiendo que acceda a `~/.config` del usuario, no del root.
