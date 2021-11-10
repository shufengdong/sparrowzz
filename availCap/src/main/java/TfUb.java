public class TfUb {
    int loadState;
    String devName;
    String mRID;
    String lineName;
    String lineMRID;
    String substation;
    double maxLoad;
    double ratedCap;

    double ub;
    double[] monthUb;

    public int getLoadState() {
        return loadState;
    }

    public void setLoadState(int loadState) {
        this.loadState = loadState;
    }

    public String getDevName() {
        return devName;
    }

    public void setDevName(String devName) {
        this.devName = devName;
    }

    public String getmRID() {
        return mRID;
    }

    public void setmRID(String mRID) {
        this.mRID = mRID;
    }

    public String getSubstation() {
        return substation;
    }

    public void setSubstation(String substation) {
        this.substation = substation;
    }

    public String getLineName() {
        return lineName;
    }

    public void setLineName(String lineName) {
        this.lineName = lineName;
    }

    public String getLineMRID() {
        return lineMRID;
    }

    public void setLineMRID(String lineMRID) {
        this.lineMRID = lineMRID;
    }

    public double getMaxLoad() {
        return maxLoad;
    }

    public void setMaxLoad(double maxLoad) {
        this.maxLoad = maxLoad;
    }

    public double getRatedCap() {
        return ratedCap;
    }

    public void setRatedCap(double ratedCap) {
        this.ratedCap = ratedCap;
    }

    public double getUb() {
        return ub;
    }

    public void setUb(double ub) {
        this.ub = ub;
    }

    public double[] getMonthUb() {
        return monthUb;
    }

    public void setMonthUb(double[] monthUb) {
        this.monthUb = monthUb;
    }
}
