package com.yukuri.uve;

import android.app.Activity;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.RectF;
import android.graphics.Typeface;
import android.os.Bundle;
import android.content.Context;
import android.view.View;

public final class MainActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(new EditorView(this));
    }

    private static final class EditorView extends View {
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);

        EditorView(Context context) {
            super(context);
            paint.setTypeface(Typeface.create("sans", Typeface.NORMAL));
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            int width = getWidth();
            int height = getHeight();
            canvas.drawColor(Color.rgb(17, 19, 24));

            paint.setColor(Color.rgb(28, 32, 40));
            canvas.drawRect(0, 0, width, 64, paint);
            canvas.drawRect(0, height - 220, width, height, paint);
            canvas.drawRect(0, 64, 260, height - 220, paint);
            canvas.drawRect(width - 300, 64, width, height - 220, paint);

            paint.setColor(Color.rgb(222, 226, 234));
            paint.setTextSize(24);
            canvas.drawText("Universal Video Editor", 24, 40, paint);
            paint.setTextSize(16);
            canvas.drawText("プロジェクト", 24, 104, paint);
            canvas.drawText("エフェクト", width - 276, 104, paint);
            paint.setColor(Color.rgb(128, 203, 196));
            canvas.drawText("メディアを読み込んで編集を開始", width / 2f - 130, height / 2f, paint);

            paint.setColor(Color.rgb(40, 45, 55));
            canvas.drawRoundRect(new RectF(24, height - 188, width - 24, height - 48), 8, 8, paint);
            paint.setColor(Color.rgb(155, 162, 176));
            canvas.drawText("タイムライン", 40, height - 158, paint);
            paint.setColor(Color.rgb(76, 86, 103));
            for (int x = 40; x < width - 40; x += 80) {
                canvas.drawRect(x, height - 130, x + 2, height - 64, paint);
            }
        }
    }
}
