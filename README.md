# 축의금 관리

결혼식 접수대에서 축의금을 기록하고, 식이 끝난 뒤 정산 내역을 확인하는 프로그램입니다. 서버나 인터넷 연결 없이 실행 파일만으로 동작합니다.

## 실행하기

### Windows
1. `wedding-gift-app.exe`를 원하는 폴더(USB도 가능)에 복사해 더블클릭합니다. 파일 이름은 바꿔도 됩니다.
2. 처음 실행하면 "이 PC는 어느 쪽 접수용인가요?"가 뜹니다. 신랑측 또는 신부측을 고릅니다.

Windows 10/11에 기본 포함된 Microsoft Edge WebView2를 사용합니다. 아주 오래된 Windows 10에서 화면이 뜨지 않으면 [WebView2 런타임](https://developer.microsoft.com/microsoft-edge/webview2/)을 설치하세요.

### macOS
1. `축의금 관리.app`을 원하는 폴더에 복사합니다. `.dmg`를 받았다면 열어서 앱을 폴더로 끌어 옵니다.
2. 서명하지 않은 앱이라 처음 실행할 때 보안 경고가 뜹니다. 아래 방법 중 하나로 실행합니다.
   - 앱을 **Control 키를 누른 채 클릭 → 열기 → 열기**
   - 또는 **시스템 설정 → 개인정보 보호 및 보안**에서 "그래도 열기"
   - 또는 터미널에서 `xattr -dr com.apple.quarantine "/경로/축의금 관리.app"`
3. 다운로드 폴더 등에서 바로 실행하면 macOS가 앱을 임시 위치에서 실행해, 앱 옆에 데이터를 저장할 수 없습니다. 이때는 데이터가 `문서/축의금` 폴더에 저장되고 화면 상단에 안내가 표시됩니다. 앱을 다른 폴더로 옮기거나 위의 `xattr` 명령을 실행하면 앱 옆에 저장됩니다.

## 데이터 파일

| 파일 | 내용 |
|---|---|
| `축의금_신랑측.json` / `축의금_신부측.json` | 각 PC의 접수 데이터 (실행 파일과 같은 폴더) |
| `축의금_통합.json` | 병합으로 만든 통합 파일 (이름은 저장할 때 바꿀 수 있음) |
| `settings.json` | PC 구분, 기본 구분, 마지막으로 연 파일, 사용 설명을 이미 봤는지 |

- 입력, 수정, 삭제할 때마다 바로 저장됩니다. 폴더째 복사하면 데이터도 함께 옮겨집니다.
- **PC 구분을 잘못 골랐을 때**는 프로그램을 끄고 `settings.json`을 지운 뒤 다시 실행하세요. 데이터 파일은 지워지지 않습니다.
- 데이터 파일은 암호화되지 않습니다. 보관에 주의하세요.

## 도움말과 사용 설명

- 화면 곳곳의 **?** 버튼을 누르면 그 기능의 설명이 나옵니다. 다시 누르거나 바깥을 누르거나 `Esc`를 누르면 닫힙니다.
- 프로그램을 실행하면 **사용 설명**(7단계)이 자동으로 열립니다. 끝까지 보거나 "다음부터 자동으로 열지 않기"를 체크하고 닫으면 다음 실행부터는 열리지 않습니다. 건너뛰기만 하면 다음 실행에 다시 나옵니다. 단, 한 번 "봤음"으로 저장된 뒤에는 건너뛰어도 다시 나오지 않습니다. 다시 나오게 하려면 `settings.json`의 `tutorialSeen`을 `false`로 바꾸거나 파일을 지우세요(파일을 지우면 신랑측/신부측 선택도 처음부터 다시 합니다).
- 오른쪽 위 **[사용 설명]** 버튼으로 언제든 다시 볼 수 있습니다. `settings.json`을 지우면 사용 설명도 처음부터 다시 나옵니다.
- 설명 문구는 `src/lib/help.ts`(? 도움말), `src/lib/tutorial.ts`(사용 설명)에서 고칠 수 있습니다.

## 당일 운영 순서

1. 신랑측 PC, 신부측 PC에서 각각 프로그램을 실행하고 PC 구분을 고릅니다.
2. **접수** 탭에서 입력합니다.
   - 이름 입력 → `Tab`(또는 `Enter`) → 금액 입력 → `Enter`로 저장
   - 금액은 만원 단위입니다. `5` = 50,000원, `5.5` = 55,000원. 빠른 버튼(5/10/15/20만)도 쓸 수 있습니다.
   - 100만원 이상은 확인 창이 뜨고, 1,000만원을 넘으면 저장되지 않습니다.
   - 신랑측/신부측 선택은 바꾸기 전까지 유지됩니다.
   - 오타는 오른쪽 최근 목록의 [수정] 버튼으로 바로 고칩니다.
3. 식이 끝나면 한쪽 PC의 데이터 파일(`축의금_신부측.json` 등)을 USB로 다른 PC에 옮깁니다.
4. **정산** 탭 → **파일 병합** → 두 파일 선택 → 통합 파일 저장 위치 선택 → **통합 파일 열기**
   - 같은 파일을 여러 번 병합해도 금액이 중복되지 않습니다.
5. 정산 탭에서 합계, 검색, 정렬, 필터를 확인하고 **CSV / 엑셀(XLSX) 내보내기**로 저장합니다.
6. 수정·삭제 기록은 **변경 이력** 탭에 남고, 삭제한 항목은 거기서 복구할 수 있습니다.

## 개발

필요한 도구: Node.js 22, Rust(stable), Windows는 Visual Studio C++ Build Tools, macOS는 Xcode Command Line Tools.

```powershell
npm ci                 # 의존성 설치 (버전 고정)
npm run tauri dev      # 개발 모드 실행
npm test               # 프론트엔드 단위 테스트
npm run check          # Svelte/TypeScript 검사
cd src-tauri; cargo test   # Rust 단위 + 통합 테스트 (먼저 npm run build 필요)
cargo test --release --test performance -- --ignored --nocapture   # 성능 측정 (1,000명)
```

### Windows 실행 파일 만들기

```powershell
npx tauri build --no-bundle
# 결과: src-tauri\target\release\wedding-gift-app.exe
```

### macOS 앱 만들기 (Mac에서 직접)

```bash
xcode-select --install                         # 처음 한 번
curl https://sh.rustup.rs -sSf | sh            # Rust 설치 (처음 한 번)
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm ci
npx tauri build --target universal-apple-darwin --bundles app,dmg
# 결과: src-tauri/target/universal-apple-darwin/release/bundle/macos/축의금 관리.app
#       src-tauri/target/universal-apple-darwin/release/bundle/dmg/*.dmg
```

### GitHub Actions로 만들기

`.github/workflows/build.yml`이 Windows exe와 macOS app/dmg를 함께 만듭니다.

1. 이 폴더를 GitHub 저장소에 올립니다.
2. 저장소의 **Actions → build → Run workflow**를 누르거나 `v1.0.0` 같은 태그를 푸시합니다.
3. 실행이 끝나면 화면 아래 **Artifacts**에서 `wedding-gift-app-windows`, `wedding-gift-app-macos`를 내려받습니다.

macOS 빌드는 아직 실제 Mac에서 검증하지 않았습니다.

## 구조

```
src/                프론트엔드 (Svelte + TypeScript)
  components/       화면 컴포넌트
  lib/              명령 래퍼, 금액 변환, 상태, 표시 형식
src-tauri/          백엔드 (Rust)
  src/model.rs      데이터 구조
  src/validation.rs 입력 검증
  src/ledger.rs     접수/수정/삭제/복구/조회/합계
  src/merge.rs      파일 병합
  src/storage.rs    파일 저장 (원자적 쓰기)
  src/export.rs     CSV/XLSX 내보내기
  src/paths.rs      저장 위치
  src/commands.rs   화면에서 호출하는 명령
aidlc-docs/         요구사항/설계 문서
```
