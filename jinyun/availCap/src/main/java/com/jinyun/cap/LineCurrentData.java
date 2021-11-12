package com.jinyun.cap;

import java.sql.Timestamp;

public class LineCurrentData {
    Timestamp data;
    double current;

    public void setData(Timestamp data) {
        this.data = data;
    }

    public Timestamp getData() {
        return data;
    }

    public void setCurrent(double current) {
        this.current = current;
    }

    public double getCurrent() {
        return current;
    }
}
