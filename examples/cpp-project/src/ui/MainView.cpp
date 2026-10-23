#include "MainView.h"

void MainView::layout() {
    setBounds(0, 0, 640, 480);
}

// @handwritten:start
void MainView::paint(Graphics& g) {
    g.fillAll(Colours::cream);
    g.drawText("Little Angel", 10, 10, 200, 24);
}
// @handwritten:end

void MainView::resized() {
    layout();
}
