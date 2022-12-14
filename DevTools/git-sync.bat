cd %~dp0
cd ..

@REM add remotes
git remote remove origin
git remote add origin git@github.com:snolab/CapsLockX.git
git remote set-url --add origin git@github.com:snomiao/CapsLockX.git
git remote set-url --add origin git@bitbucket.org:snomiao/capslockx.git
git remote set-url --add origin git@gitlab.com:snomiao/CapsLockX.git
git remote set-url --add origin git@gitee.com:snomiao/CapslockX.git
git remote -v

@REM sync
git fetch origin
git pull --set-upstream origin main
git push origin main --follow-tags
