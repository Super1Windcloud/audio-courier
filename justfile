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
    just push &&  git push repo  main:release


clean :
   cd  src-tauri &&  cargo clean

upgrade:
    cd src-tauri &&  cargo update && cargo upgrade
