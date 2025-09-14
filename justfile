# https://just.systems

default:
    echo 'Hello, world!'


push :
   git add . && git commit -m "update" && git push repo main


clear :
  git  rm --cached -r .

  

release:
    just push &&  git push repo  main:release s