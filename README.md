# mttr
## 使い方 (CentOS7の場合)
### 1. リポジトリをクローンする
```
$ git clone
```
### 2. リポジトリに移動する
```
$ cd mttr
```
### 3. コンパイラとパッケージマネージャをインストールする
```
$ yum install gcc
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
```
### 4. コンパイルする
```
$ cargo build
```
### 5. 実行する
```
$ target/debug/mttr [ファイル名] [N] [m] [t]
```
****
## 結果の確認
結果は`result/`に出力されます
````
ttr_server.txt -> N回以上タイムアウトしている期間(設問1・設問2)
overload.txt -> m回の平均応答時間がt以上の期間(設問3)
ttr_subnet.txt -> サブネット内のすべてのサーバーがn回以上タイムアウトしている期間(設問4)
````