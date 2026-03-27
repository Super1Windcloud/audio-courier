# https://just.systems

default:
    echo 'Hello, world!'


push :
   git add . && git commit -m "update" && git push repo main


clear :
  git  rm --cached -r . && just push

pull :
   git pull repo main

release:
    pnpm release


clean :
   cd  src-tauri &&  cargo clean

upgrade:
    cd src-tauri &&  cargo update && cargo upgrade


nsis:
    pnpm tb && mv  src-tauri/target/release/bundle/nsis  ./bundle


