package com.tessera.example;

import android.app.assist.AssistStructure;
import android.content.ComponentName;
import android.os.CancellationSignal;
import android.service.autofill.AutofillService;
import android.service.autofill.Dataset;
import android.service.autofill.FillCallback;
import android.service.autofill.FillContext;
import android.service.autofill.FillRequest;
import android.service.autofill.FillResponse;
import android.service.autofill.SaveCallback;
import android.service.autofill.SaveInfo;
import android.service.autofill.SaveRequest;
import android.util.Log;
import android.view.autofill.AutofillId;
import android.view.autofill.AutofillValue;
import android.widget.RemoteViews;

import java.util.List;

public final class PlatformViewAutofillService extends AutofillService {
    @Override
    public void onFillRequest(FillRequest request, CancellationSignal cancellation,
            FillCallback callback) {
        List<FillContext> contexts = request.getFillContexts();
        AssistStructure structure = contexts.get(contexts.size() - 1).getStructure();
        ComponentName activity = structure.getActivityComponent();
        if (activity == null || !"com.tessera.example".equals(activity.getPackageName())) {
            callback.onSuccess(null);
            return;
        }
        for (int index = 0; index < structure.getWindowNodeCount(); index++) {
            AssistStructure.ViewNode editor = findEditor(structure.getWindowNodeAt(index).getRootViewNode());
            if (editor == null) continue;
            RemoteViews label = new RemoteViews(getPackageName(), android.R.layout.simple_list_item_1);
            label.setTextViewText(android.R.id.text1, "Tessera test autofill");
            Dataset dataset = new Dataset.Builder(label)
                    .setValue(editor.getAutofillId(), AutofillValue.forText("Hello"))
                    .build();
            callback.onSuccess(new FillResponse.Builder().addDataset(dataset)
                    .setSaveInfo(new SaveInfo.Builder(SaveInfo.SAVE_DATA_TYPE_USERNAME,
                            new AutofillId[] {editor.getAutofillId()})
                            .setFlags(SaveInfo.FLAG_DONT_SAVE_ON_FINISH).build())
                    .build());
            Log.i("TesseraAutofillProbe", "native editor present in autofill request");
            return;
        }
        Log.i("TesseraAutofillProbe", "native editor missing from autofill request");
        callback.onSuccess(null);
    }

    private static AssistStructure.ViewNode findEditor(AssistStructure.ViewNode view) {
        if ("Native editor".equals(view.getHint()) && view.getAutofillId() != null) {
            return view;
        }
        for (int index = 0; index < view.getChildCount(); index++) {
            AssistStructure.ViewNode editor = findEditor(view.getChildAt(index));
            if (editor != null) return editor;
        }
        return null;
    }

    @Override
    public void onSaveRequest(SaveRequest request, SaveCallback callback) {
        List<FillContext> contexts = request.getFillContexts();
        AssistStructure structure = contexts.get(contexts.size() - 1).getStructure();
        for (int index = 0; index < structure.getWindowNodeCount(); index++) {
            AssistStructure.ViewNode editor = findEditor(structure.getWindowNodeAt(index).getRootViewNode());
            AutofillValue value = editor == null ? null : editor.getAutofillValue();
            if (value != null && value.isText() && "HelloSaved".contentEquals(value.getTextValue())) {
                Log.i("TesseraAutofillProbe", "received edited probe value in save request");
                callback.onSuccess();
                return;
            }
        }
        callback.onFailure("Edited probe value missing");
    }
}
