public class LineParamData {
    String devName;
    String pole1;
    String pole2;
    double ratedI;
    int type;   // 1为JK，2为LGJ

    public void setDevName(String devName) {
        this.devName = devName;
    }

    public String getDevName() {
        return devName;
    }

    public void setPole1(String pole1) {
        this.pole1 = pole1;
    }

    public String getPole1() {
        return pole1;
    }

    public void setPole2(String pole2) {
        this.pole2 = pole2;
    }

    public String getPole2() {
        return pole2;
    }

    public void setRatedI(double ratedI) {
        this.ratedI = ratedI;
    }

    public double getRatedI() {
        return ratedI;
    }

    public void setType(int type) {
        this.type = type;
    }

    public int getType() {
        return type;
    }
}
