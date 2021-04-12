@echo off
chcp 65001
REM [Working with Git remotes and pushing to multiple Git repositories | Jigarius.com]( https://jigarius.com/blog/multiple-git-remote-repositories )
git remote remove remote
git remote add remote git@github.com:snomiao/CapsLockX.git
git remote set-url --add --push remote git@github.com:snomiao/CapsLockX.git
git remote set-url --add --push remote git@gitlab.com:snomiao/CapsLockX.git
git remote set-url --add --push remote git@gitee.com:snomiao/CapslockX.git
git remote set-url --add --push remote git@bitbucket.org:snomiao/capslockx.git
git fetch --all 
git push --set-upstream remote master
git push
