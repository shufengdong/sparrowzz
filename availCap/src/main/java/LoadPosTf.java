public class LoadPosTf {
    String canIn;
    String tfName;   // 最优接入位置公变的mRID
    String tfMRID;   // 最优接入位置公变的名称
    String phase; // 公变接入相别
    double[] newLoad;   // 接入的负荷
    double[] tfOrgLoad;   // 接入点公变原始负荷
    double[] tfNewLoad; // 接入后公变负荷
    double[] tfRateCap;   // 接入点公变额定容量

    public String getCanIn() {
        return canIn;
    }

    public void setCanIn(String canIn) {
        this.canIn = canIn;
    }

    public String getTfName() {
        return tfName;
    }

    public void setTfName(String tfName) {
        this.tfName = tfName;
    }

    public String getTfMRID() {
        return tfMRID;
    }

    public void setTfMRID(String tfMRID) {
        this.tfMRID = tfMRID;
    }

    public String getPhase() {
        return phase;
    }

    public void setPhase(String phase) {
        this.phase = phase;
    }

    public double[] getNewLoad() {
        return newLoad;
    }

    public void setNewLoad(double[] newLoad) {
        this.newLoad = newLoad;
    }

    public double[] getTfOrgLoad() {
        return tfOrgLoad;
    }

    public void setTfOrgLoad(double[] tfOrgLoad) {
        this.tfOrgLoad = tfOrgLoad;
    }

    public double[] getTfNewLoad() {
        return tfNewLoad;
    }

    public void setTfNewLoad(double[] tfNewLoad) {
        this.tfNewLoad = tfNewLoad;
    }

    public double[] getTfRateCap() {
        return tfRateCap;
    }

    public void setTfRateCap(double[] tfRateCap) {
        this.tfRateCap = tfRateCap;
    }
}
