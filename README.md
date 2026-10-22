# conexus-drivers

Módulos de driver por marca e aparelho do [Conexus Mobile OS](https://github.com/conexus-systems/conexus-mobile-os). Cada módulo vira um bundle (ADR-014) que o `conexus-driver-manager` só instala se estiver assinado.

## Regras

- **Nenhum blob de fabricante neste repositório** (ADR-014). Os arquivos de `/vendor` vêm do aparelho de quem instala (`conexus-installer extract`) e ficam fora do git. O `.gitignore` bloqueia `*.ko`, `*.so`, `*.bin`, `*.img`, `*.apk` e `*.zip`, e o teste `no_blobs` falha se algum entrar.
- **Nenhum módulo declara suporte sem evidência.** `Support::Supported` só depois de teste em aparelho com os critérios C1–C5 do ADR-010. `Untested` significa que a lógica passa em testes contra uma árvore de arquivos falsa e **nunca rodou num aparelho**.
- Linguagem: Rust (ADR-016).

## Módulos

| Módulo | Aparelho | Estado |
|---|---|---|
| `modules/google-akita` | Pixel 8a | **Esqueleto.** 3 componentes (vibrator, thermal, power) com manifest válido e `Unsupported`. Sem HAL real |
| `modules/generic-linux` | Qualquer kernel Linux/Android | **Lógica pronta, nunca rodou em aparelho** (`Untested`). Térmico (`/sys/class/thermal`) e fontes de energia (`/sys/class/power_supply`), sem blobs. Falta o HAL AIDL e o teste em aparelho |
| `modules/common` | — | Tipos comuns (`Support`: `Unsupported` < `Untested` < `Supported`) |

TCL 505 e demais aparelhos entram quando houver inventário e extração do aparelho real (`docs/devices/tcl-505.md` no conexus-mobile-os).

## Desenvolvimento

```bash
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test
```

A dependência `conexus-core` aponta para um commit fixo do `conexus-mobile-os` (repositório privado, ramo `feat/vendor-extract`, ainda sem merge na `main`). Quando o conexus-mobile-os for mesclado, trocar o `rev` por um commit da `main`. O CI exige o segredo `MOBILE_OS_READ_TOKEN` (leitura do conexus-mobile-os).
