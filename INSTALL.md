# Instrucciones de instalación

## Linux

### Compilación desde código fuente

**Activar el daemon:**

```bash
sudo systemctl enable vasak-impulse-daemon
sudo systemctl start vasak-impulse-daemon
```

**Ver logs:**

```bash
sudo journalctl -u vasak-impulse-daemon -f
```

**Crear configuración de usuario:**

```bash
mkdir -p ~/.config/vasak
cp /etc/vasak/shortcut.example.json ~/.config/vasak/shortcut.json
# Editar según sea necesario
nano ~/.config/vasak/shortcut.json
```

### Permisos sin sudo (opcional)

Para ejecutar el daemon sin necesidad de `sudo`, agrega tu usuario al grupo `input`:

```bash
sudo usermod -aG input $USER
```

Luego cierra sesión e inicia sesión nuevamente. Con las reglas udev instaladas, el daemon tendrá acceso a `/dev/input/`.

```bash
git clone https://github.com/VasakOS/vasak-impulse-daemon.git
cd vasak-impulse-daemon
cargo build --release
```

## Desinstalación

### Desinstalación manual

```bash
sudo systemctl stop vasak-impulse-daemon
sudo systemctl disable vasak-impulse-daemon
sudo rm /usr/bin/vasak-impulse-daemon
sudo rm /usr/lib/systemd/system/vasak-impulse-daemon.service
sudo rm /etc/udev/rules.d/99-vasak-impulse-daemon.rules
sudo udevadm control --reload
sudo systemctl daemon-reload
```

## Solución de problemas

### "Permission denied" al acceder a `/dev/input`

**Opción 1**: Usar `sudo`

```bash
sudo systemctl start vasak-impulse-daemon
```

**Opción 2**: Agregar usuario al grupo `input` (requiere reiniciar sesión)

```bash
sudo usermod -aG input $USER
# Cierra e inicia sesión nuevamente
```

### El daemon no detecta el archivo de configuración

Verifica que el archivo existe en la ubicación correcta:

```bash
ls -la ~/.config/vasak/shortcut.json
```

Si no existe, el daemon debería crearlo automáticamente. Si sigue sin funcionar, verifica los logs:

```bash
sudo journalctl -u vasak-impulse-daemon -n 50
```

### El daemon se detiene después de un rato

Verifica si hay errores en los logs:

```bash
sudo journalctl -u vasak-impulse-daemon -f
```

Si es por falta de configuración, asegúrate que el JSON es válido:

```bash
jq . ~/.config/vasak/shortcut.json
```
