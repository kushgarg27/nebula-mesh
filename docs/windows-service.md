# Windows Service Setup (Broker on Ideapad)

## Build
- Build broker binary on Windows: `cargo build --release -p broker`
- Copy executable path, e.g. `C:\nebula\broker.exe`

## Register service
Run PowerShell as Administrator:

```powershell
sc.exe create NebulaBroker binPath= "C:\nebula\broker.exe" start= auto
sc.exe description NebulaBroker "Nebula Mesh WebSocket broker"
sc.exe start NebulaBroker
```

## Firewall allow rule
```powershell
New-NetFirewallRule -DisplayName "Nebula Broker 24800" -Direction Inbound -Action Allow -Protocol TCP -LocalPort 24800
```

## Verify
```powershell
sc.exe query NebulaBroker
netstat -ano | findstr 24800
```
