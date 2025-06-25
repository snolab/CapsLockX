cd %~dp0
cd ..

@REM add remotes
git remote remove all
git remote add all git@github.com:snolab/CapsLockX.git
git remote set-url --add all git@github.com:snomiao/CapsLockX.git
git remote set-url --add all git@bitbucket.org:snomiao/capslockx.git
git remote set-url --add all git@gitlab.com:snomiao/CapsLockX.git
git remote set-url --add all git@gitee.com:snomiao/CapslockX.git
git remote -v

@REM sync
git fetch all
git pull --set-upstream all main
git push all main --follow-tags
