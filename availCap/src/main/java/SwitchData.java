import java.sql.Timestamp;

public class SwitchData {
    String devName;
    String mRID;
    Timestamp data;
    double Ia;
    double Ib;
    double Ic;

    public void setDevName(String devName) {
        this.devName = devName;
    }

    public String getDevName() {
        return devName;
    }

    public void setmRID(String mRID) {
        this.mRID = mRID;
    }

    public String getmRID() {
        return mRID;
    }

    public void setData(Timestamp data) {
        this.data = data;
    }

    public Timestamp getData() {
        return data;
    }

    public void setIa(double ia) {
        Ia = ia;
    }

    public double getIa() {
        return Ia;
    }

    public void setIb(double ib) {
        Ib = ib;
    }

    public double getIb() {
        return Ib;
    }

    public void setIc(double ic) {
        Ic = ic;
    }

    public double getIc() {
        return Ic;
    }
}
