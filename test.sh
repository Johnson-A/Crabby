mem=$(free|awk '/^Mem:/{print $4}')
nthreads=$(nproc)
memper=$((mem*4/5/nthreads/1024/2))
echo $memper
output=/home/alex/Dropbox/workspace/rust/chess/results/games.pgn
book=/home/alex/Dropbox/workspace/rust/chess/info/ecoe.pgn

cutechess-cli -fcp cmd=$1 proto=uci initstr="setoption name hash value $memper" \
-scp cmd=$2 proto=uci initstr="setoption name hash value $memper" \
-both tc=40/60 timemargin=10000000 \
-games 2000 -repeat -recover \
-concurrency $nthreads -pgnin $book -pgnout $output
