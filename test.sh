mem=$(free|awk '/^Mem:/{print $4}')
nthreads=$(nproc)
memper=$((mem*4/5/nthreads/1024))
echo $memper
output=/home/alex/Dropbox/workspace/rust/chess/results/games.pgn
book=/home/alex/Dropbox/workspace/rust/chess/info/ecoe.pgn

cutechess-cli -fcp cmd=crabby proto=uci initstr="setoption name hash value $memper" \
-scp cmd=fairymax proto=xboard \
-both tc=40/60 timemargin=10000000 book=$book \
-games 200 -repeat -recover \
-concurrency $nthreads -pgnout $output
