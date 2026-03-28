set export := true

CN_API_KEY := "cn_NZ6JCmZfDfj0NZ80ta60ZqHujE7XjEH6DQojTmp7q5IUIOIJnSgelOvoEoLNrUNnIwDEuaARGIJEMT2ZCpP1Ag"
TAURI_SIGNING_PRIVATE_KEY := "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5bXJta3ZwcC9qdmg5T3JyOSsrUi9OenlabzBOT2VDNmszcXd3WDJPWUp5NEFBQkFBQUFBQUFBQUFBQUlBQUFBQUpLbHJIVGpyc2FjUDY4b3lmemVBT2F2SkNFRm5RNDQvY0V3V1pGN1A3K2IwbTJsVEZPS080QkZydXJjc1NHR3dBbHM5NHVCRXZXaXRmRjkrR3JqZHdQcm92YXZZeTRlL3E5SHR5bzdQRHVvSndDcTBxVnBuV04wTkJLSHZ1MkpzRVdVcFpiWW9TSnM9Cg=="
TAURI_SIGNING_PRIVATE_KEY_PASSWORD := "superwindcloud"


default:
    echo 'Hello, world!'


push :
   git add . && git commit -m "update" && git push repo main


clear :
  git  rm --cached -r . && just push

pull :
   git pull repo main


release:
    pnpm bump
    pnpm release

publish:
    pnpm release

build:
    pnpm release:build

apple_arm:
    RELEASE_TAURI_ARGS="--target aarch64-apple-darwin" just release

clean:
   cd  src-tauri &&  cargo clean

upgrade:
    cd src-tauri &&  cargo update && cargo upgrade


nsis:
    pnpm tb && mv  src-tauri/target/release/bundle/nsis  ./bundle
