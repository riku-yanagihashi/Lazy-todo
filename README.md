# Lazy Todo

## インストール方法

### MacOSの場合
RUSTの実行環境が整っている状態で以下のコマンドをターミナルで入力してください
```sh
git clone --depth 1  https://github.com/riku-yanagihashi/Lazy-todo.git ~/Lazy-todo && cd Lazy-todo/todo_cli && cargo build && cargo install --path . && cd 
```

※ Windowsはサポートする予定ありません

## 使用方法

### 基本的な操作
j:カーソルを下に移動 /n
k:カーソルを上に移動 /n
q:アプリを終了など /n
a:タスクの追加 /n
e:カーソル上のタスク編集モードになる /n
d:カーソル上のタスクを削除 /n
l:タスクの詳細を表示 /n
Enter:完了済みかどうかを変更 /n
Esc:ノーマルモードに戻る /n

### タスク追加

1. aキーを押してタスク追加モードへ移行する
2. タイトルを入力し、Enterで決定
3. 内容を入力し、Enterで決定(未記入でも可)
4. 優先度をj/kキーで選択し、Enterで決定
5. 期限を設定し、Enterキーで決定
6. 追加完了

### タスク編集

1. eキーを押して編集モードに移行(カーソル上のタスク)
2. titleを編集するか、編集しないならEnterで次に進む
3. 内容を編集するか、Enterで次に進む
4. 優先度をj/kキーで編集し、Enterで次に進む
5. 期限を編集し、Enterで決定

## アンインストール方法
```sh
cargo uninstall ltd
```







