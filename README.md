# Universal Video Editor

Windows / Android / iOSで動作する、AviUtl・YMM4相当の動画編集ツールです。

## 方針

- 編集エンジンはプラットフォーム共通のRustコアとして実装する
- UIはコアと分離し、Windows・Android・iOSで同じ編集モデルを利用する
- まずUVE独自のエフェクト、移動制御、PSDレイヤー表現を実装する
- 次にAviUtl系スクリプト互換、将来的にプラグイン互換を追加する

## 現在のコア

`crates/uve-core` に以下の基礎機能を実装しています。

- 数値キーフレームと線形・ステップ補間
- エフェクト定義とパラメータ
- PSDドキュメント・レイヤーの共通表現
- PSD実ファイルのヘッダー検証と基本メタデータ読み込み
- AviUtl系スクリプトの最小コマンドパーサー
- プロジェクト、タイムライン、クリップ、トランスフォームのJSON入出力
- Android/NDK接続用のC ABIプロジェクトブリッジ
- 編集状態のUndo/Redo履歴

画像・PSDの実ファイル読み込み、動画デコーダー、レンダラー、各OS向けUIは次の段階で追加します。

## Android

Android版はルートのGradleプロジェクトから `:android:app` としてビルドします。
AviUtlの操作感をスマートフォン向けに最適化して再現する方針です。
現在は編集画面のUIシェルを搭載しており、次にRustコアとのJNI/UniFFI接続を追加します。

```bash
./gradlew assembleDebug
```

互換検証用の素材は `sample_data/` に配置します。
