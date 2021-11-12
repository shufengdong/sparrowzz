package com.jinyun.cap;

import java.io.File;
import java.io.IOException;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.sql.*;
import java.util.HashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;

public class SqliteDb {

    String dbFile;

    public SqliteDb(String dbFile) {
        this.dbFile = dbFile;
    }

    private Connection createConn() {
        File f = new File(dbFile);
        if(!f.exists()) {
            try {
                f.createNewFile();
            } catch (IOException e) {
                e.printStackTrace();
            }
        }
        Connection conn;
        try {
            Class.forName("org.sqlite.JDBC");
            conn = DriverManager.getConnection("jdbc:sqlite:"+dbFile);
        } catch (Exception e) {
            StringWriter w = new StringWriter();
            e.printStackTrace(new PrintWriter(w, true));
            System.out.println("111"+e.getMessage());
            return null;
        }
        return conn;
    }

    public Boolean executeSql(String sql) {
        Connection conn = createConn();
        if(conn == null) return false;
        Statement stmt =  null;
        try {
            conn.setAutoCommit(false);
            stmt = conn.createStatement();
            stmt.addBatch(sql);
            stmt.executeBatch();
            conn.commit();
        } catch (SQLException e) {
            try {
                conn.rollback();
            } catch (SQLException e1) {
                System.out.println(e1.getMessage());
            }
            StringWriter w =new StringWriter();
            e.printStackTrace(new PrintWriter(w, true));
            System.out.println(e.getMessage());
            return false;
        } finally {
            try {
                stmt.close();
                conn.setAutoCommit(true);
                conn.close();
            } catch (SQLException e) {
            }
        }
        return true;
    }

    public Boolean executeSqls(List<String> sqls) {
        Connection conn = createConn();
        if(conn == null) return false;
        Statement stmt =  null;
        try {
            conn.setAutoCommit(false);
            stmt = conn.createStatement();
            for (String obj : sqls)
                stmt.addBatch(obj);
            stmt.executeBatch();
            conn.commit();
        } catch (SQLException e) {
            try {
                conn.rollback();
            } catch (SQLException e1) {
                System.out.println(e1.getMessage());
            }
            StringWriter w =new StringWriter();
            e.printStackTrace(new PrintWriter(w, true));
            System.out.println(e.getMessage());
            return false;
        } finally {
            try {
                stmt.close();
                conn.setAutoCommit(true);
                conn.close();
            } catch (SQLException e) {
            }
        }
        return true;
    }

    public void initDb(String initSql) {
        Connection conn = createConn();
        if (conn == null) {
            System.out.println("Can't connect to sqlite !!!");
            return;
        }
        Statement stmt= null;
        try {
            stmt = conn.createStatement();
            stmt.executeUpdate(initSql);
        } catch (SQLException e ) {
            if (e.getMessage().contains("table $tableName already exists")) {
                System.out.println("Table $tableName already exists");
            } else {
                StringWriter w = new StringWriter();
                e.printStackTrace(new PrintWriter(w, true));
                System.out.println(e.getMessage());
            }
        } finally {
            try {
                stmt.close();
                conn.close();
            } catch (SQLException e) {
            }
        }
    }

    public List<String> getTableNames() {
        Connection conn = createConn();
        String sql = "select name from sqlite_master  where type='table'";
        Statement stmt = null;
        ResultSet rs = null;
        List<String> tableNames = new LinkedList<>();
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                tableNames.add(rs.getString("name"));
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return tableNames;
    }

    public boolean existTable(String table) {
        Connection conn = createConn();
        String sql = "select count(*) as tableNum from sqlite_master  where type='table' and name='" + table + "'";
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                if (rs.getInt("tableNum") > 0)
                    return true;
                else
                    return false;
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return false;
    }

    public List<String> queryMRIds(String tableName) {
        List<String> mRIDs = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select mRID from " + tableName + " group by mRId";
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                mRIDs.add(rs.getString("mRID"));
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return mRIDs;
    }

    /**
     * 查询开关对应的设备，或设备对应的开关，或公变对应的线路
     * @param tableName
     * @return
     */
    public List<String[]> querySwToDev(String tableName, String mRID) {
        List<String[]> mRIDNames = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select devName2,mRID2 from " + tableName + " where mRID1='" + mRID + "'";
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                String[] mRIDName = new String[2];
                mRIDName[0] = rs.getString("devName2");
                mRIDName[1] = rs.getString("mRID2");
                mRIDNames.add(mRIDName);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return mRIDNames;
    }

    public List<Object> queryData(String tableName, String psId) {
        List<Object> objs = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where psId='" + psId + "'";
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                Object obj = createObj(tableName, rs);
                objs.add(obj);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return objs;
    }

    /**
     * 查询变电站名称
     * @param tableName 表名
     * @return
     */
    public String queryFeederName(String tableName) {
        String feeder = null;
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                feeder = rs.getString("feeder");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return feeder;
    }

    /**
     * 查询变电站名称
     * @param tableName 表名
     * @return
     */
    public String querySubstationName(String tableName) {
        String substation = null;
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                substation = rs.getString("substation");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return substation;
    }

    /**
     * 按季节查询主线电流数据
     * @param tableName 表名
     * @param season 季节
     * @return
     */
    public List<LineCurrentData> queryLineCurrentData(String tableName, int season) {
        List<LineCurrentData> lineCurrentDatas = new LinkedList<>();
        Connection conn = createConn();
        String sql;
        if (season == 1)
            sql = "select * from " + tableName + " where strftime('%m',date) >= '01' and strftime('%m',date) <= '03'";
        else if (season == 2)
            sql = "select * from " + tableName + " where strftime('%m',date) >= '04' and strftime('%m',date) <= '06'";
        else if (season == 3)
            sql = "select * from " + tableName + " where strftime('%m',date) >= '07' and strftime('%m',date) <= '09'";
        else if (season == 4)
            sql = "select * from " + tableName + " where strftime('%m',date) >= '10' and strftime('%m',date) <= '12'";
        else
            return lineCurrentDatas;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                LineCurrentData lineCurrentData = new LineCurrentData();
                lineCurrentData.setData(rs.getTimestamp("date"));
                lineCurrentData.setCurrent(rs.getDouble("current"));
                lineCurrentDatas.add(lineCurrentData);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return lineCurrentDatas;
    }

    /**
     * 按季节查询主线最大电流数据
     * @param tableName 表名
     * @param season 季节
     * @param pointNum 一天的点数
     * @return 一天的电流数据
     */
    public double[] querySeasonLineI(String tableName, int season, int pointNum) {
        double[] seasonLineI = new double[pointNum];
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where season=" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            int count = 0;
            while (rs.next() && count < pointNum) {
                seasonLineI[count] = rs.getDouble("current");
                count++;
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return seasonLineI;
    }

    /**
     * 按mRID和季节查询开关数据
     * @param tableName 表名
     * @param mRID
     * @param season 季节
     * @return
     */
    public List<SwitchData> querySwitchData(String tableName, String mRID, int season) {
        List<SwitchData> swDatas = new LinkedList<>();
        Connection conn = createConn();
        String sql;
        if (season == 1)
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) >= '01' and strftime('%m',date) <= '03'";
        else if (season == 2)
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) >= '04' and strftime('%m',date) <= '06'";
        else if (season == 3)
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) >= '07' and strftime('%m',date) <= '09'";
        else if (season == 4)
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) >= '10' and strftime('%m',date) <= '12'";
        else
            return swDatas;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                SwitchData swData = new SwitchData();
                swData.setDevName(rs.getString("devName"));
                swData.setmRID(mRID);
                swData.setData(rs.getTimestamp("date"));
                swData.setIa(rs.getDouble("Ia"));
                swData.setIb(rs.getDouble("Ib"));
                swData.setIc(rs.getDouble("Ic"));
                swDatas.add(swData);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return swDatas;
    }

    /**
     * 按mRID和月份查询开关数据
     * @param tableName 表名
     * @param mRID
     * @param month 月份
     * @return
     */
    public List<SwitchData> querySwDataByMonth(String tableName, String mRID, int month) {
        List<SwitchData> swDatas = new LinkedList<>();
        Connection conn = createConn();
        String sql;
        if (month >= 1 && month <= 12) {
            String monthStr = String.format("%02d", month);
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) = '" + monthStr + "'";
        } else
            return swDatas;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                SwitchData swData = new SwitchData();
                swData.setDevName(rs.getString("devName"));
                swData.setmRID(mRID);
                swData.setData(rs.getTimestamp("date"));
                swData.setIa(rs.getDouble("Ia"));
                swData.setIb(rs.getDouble("Ib"));
                swData.setIc(rs.getDouble("Ic"));
                swDatas.add(swData);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return swDatas;
    }

    /**
     * 按mRID和季节查询开关最大电流或聚类结果
     * @param tableName 表名
     * @param mRID mRID
     * @param season 季节
     * @return
     */
    public double[] querySeasonSwitchI(String tableName, String mRID, int season, int pointNum) {
        double[] seasonSwI = new double[pointNum];
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "' and season =" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            int count = 0;
            while (rs.next() && count < pointNum) {
                seasonSwI[count] = rs.getDouble("Ia");
                count++;
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) {
                    rs.close();
                }
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return seasonSwI;
    }

    /**
     * 按mRID查询开关最大电流
     * @param tableName 表名
     * @param mRID mRID
     * @return
     */
    public double queryMaxSwitchI(String tableName, String mRID, int season) {
        double maxSwI = 10;
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "' and season =" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            int count = 0;
            while (rs.next()) {
                maxSwI = rs.getDouble("Ia");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) {
                    rs.close();
                }
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return maxSwI;
    }

    /**
     * 按mRID和季节查询公变数据
     * @param tableName 表名
     * @param mRID
     * @param season 季节
     * @return
     */
    public List<TFData> queryTFData(String tableName, String mRID, int season) {
        List<TFData> tfDatas = new LinkedList<>();
        Connection conn = createConn();
        String sql;
        if (season == 1)
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) >= '01' and strftime('%m',date) <= '03'";
        else if (season == 2)
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) >= '04' and strftime('%m',date) <= '06'";
        else if (season == 3)
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) >= '07' and strftime('%m',date) <= '09'";
        else if (season == 4)
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) >= '10' and strftime('%m',date) <= '12'";
        else
            return tfDatas;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                TFData tfData = new TFData();
                tfData.setDevName(rs.getString("devName"));
                tfData.setmRID(mRID);
                tfData.setDate(rs.getTimestamp("date"));
                tfData.setIa(rs.getDouble("Ia"));
                tfData.setIb(rs.getDouble("Ib"));
                tfData.setIc(rs.getDouble("Ic"));
                tfData.setUa(rs.getDouble("Ua"));
                tfData.setUb(rs.getDouble("Ub"));
                tfData.setUc(rs.getDouble("Uc"));
                tfDatas.add(tfData);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return tfDatas;
    }

    /**
     * 按mRID和月份查询公变数据
     * @param tableName 表名
     * @param mRID
     * @param month 季节
     * @return
     */
    public List<TFData> queryTFDataByMonth(String tableName, String mRID, int month) {
        List<TFData> tfDatas = new LinkedList<>();
        Connection conn = createConn();
        String sql;
        if (month >= 1 && month <= 12) {
            String monthStr = String.format("%02d", month);
            sql = "select * from " + tableName + " where mRID='" + mRID + "' and strftime('%m',date) = '" + monthStr + "'";
        } else
            return tfDatas;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                TFData tfData = new TFData();
                tfData.setDevName(rs.getString("devName"));
                tfData.setmRID(mRID);
                tfData.setDate(rs.getTimestamp("date"));
                tfData.setIa(rs.getDouble("Ia"));
                tfData.setIb(rs.getDouble("Ib"));
                tfData.setIc(rs.getDouble("Ic"));
                tfData.setUa(rs.getDouble("Ua"));
                tfData.setUb(rs.getDouble("Ub"));
                tfData.setUc(rs.getDouble("Uc"));
                tfDatas.add(tfData);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return tfDatas;
    }

    /**
     * 按mRID查询公变最大负荷、最大电流
     * @param tableName 表名
     * @param mRID mRID
     * @return
     */
    public double queryMaxTFP(String tableName, String mRID, int season) {
        double maxTFP = 50000;
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "' and season =" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                maxTFP = rs.getDouble("P");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) {
                    rs.close();
                }
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return maxTFP;
    }

    /**
     * 按mRID查询公变三相不平衡度
     * @param tableName 表名
     * @param mRID mRID
     * @param season 季节
     * @return
     */
    public double[] queryTFUb(String tableName, String mRID, int season) {
        double[] ubIU = new double[2];
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "' and season =" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                ubIU[0] = rs.getDouble("ubI2");
                ubIU[1] = rs.getDouble("ubV2");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) {
                    rs.close();
                }
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return ubIU;
    }

    /**
     * 按mRID查询公变负荷接入相别
     * @param tableName 表名
     * @param mRID mRID
     * @param season 季节
     * @return
     */
    public String queryMinIPhase(String tableName, String mRID, int season) {
        int phaseInt = 0;
        String phase = "a";
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "' and season =" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                phaseInt = rs.getInt("phase");
            }
            if (phaseInt == 1)
                phase = "b";
            if (phaseInt == 2)
                phase = "c";
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) {
                    rs.close();
                }
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return phase;
    }

    /**
     * 查询设备名称、mRID
     * @param tableName 表名
     * @return
     */
    public Map<String, String> queryNameToMRID(String tableName) {
        Map<String, String> nameToMRID = new HashMap<>();
        Connection conn = createConn();
        String sql = "select devName, mRID from " + tableName + " group by devName";
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                nameToMRID.put(rs.getString("devName"), rs.getString("mRID"));
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return nameToMRID;
    }

    /**
     * 查询公变容量
     * @param tableName 表名
     * @return
     */
    public double queryTFCap(String tableName, String mRID) {
        double cap = 400;
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "'";
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                cap = rs.getDouble("ratedCap");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return cap;
    }

    /**
     * 按mRID和季节查询公变最大负荷或聚类结果
     * @param tableName 表名
     * @param mRID mRID
     * @param season 季节
     * @return
     */
    public double[] querySeasonTFP(String tableName, String mRID, int season, int pointNum) {
        double[] seasonSwI = new double[pointNum];
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "' and season =" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            int count = 0;
            while (rs.next() && count < pointNum) {
                seasonSwI[count] = rs.getDouble("P");
                count++;
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) {
                    rs.close();
                }
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return seasonSwI;
    }

    /**
     * 按mRID和季节查询公变最大三相不平衡度或聚类结果
     * @param tableName 表名
     * @param mRID mRID
     * @param season 季节
     * @return
     */
    public double[] querySeasonTFUb(String tableName, String mRID, int season, int pointNum) {
        double[] seasonSwI = new double[pointNum];
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "' and season =" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            int count = 0;
            while (rs.next() && count < pointNum) {
                seasonSwI[count] = rs.getDouble("ubI2");
                count++;
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) {
                    rs.close();
                }
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return seasonSwI;
    }

    /**
     * 查询线路电流限值
     * @param tableName 表名
     * @return
     */
    public List<LineParamData> queryLineParam(String tableName) {
        List<LineParamData> lineParamDatas = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                LineParamData lineParamData = new LineParamData();
                lineParamData.setDevName(rs.getString("devName"));
                lineParamData.setPole1(rs.getString("pole1"));
                lineParamData.setPole2(rs.getString("pole2"));
                lineParamData.setRatedI(rs.getDouble("ratedCurrent"));
                lineParamData.setType(rs.getInt("type"));
                lineParamDatas.add(lineParamData);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return lineParamDatas;
    }

    /**
     * 查询单线电流限值
     * @param tableName 表名
     * @return
     */
    public double queryOneLineParam(String tableName, String mRID) {
        double oneLineParam = 1000;
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "'";
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                oneLineParam = rs.getDouble("ratedCurrent");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return oneLineParam;
    }

    /**
     * 按mRID和季节查询开关或公变可接入容量
     * @param tableName 表名
     * @param mRID mRID
     * @param season 季节
     * @return
     */
    public double[] queryAvailCap(String tableName, String mRID, int season, int pointNum) {
        double[] availCap = new double[pointNum];
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where mRID='" + mRID + "' and season =" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            int count = 0;
            while (rs.next() && count < pointNum) {
                availCap[count] = rs.getDouble("availCap");
                count++;
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                if (rs != null) {
                    rs.close();
                }
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return availCap;
    }

    /**
     * 查询预警线路
     * @param tableName 表名
     * @return
     */
    public List<WarnLine> queryWarnLine(String tableName) {
        List<WarnLine> warnLines = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                WarnLine warnLine = new WarnLine();
                warnLine.setLoadState(rs.getInt("loadState"));
                warnLine.setDevName(rs.getString("devName"));
                warnLine.setmRID(rs.getString("mRID"));
                warnLine.setSubstation(rs.getString("substation"));
                warnLine.setSwitchName(rs.getString("switchName"));
                warnLine.setSwitchMRID(rs.getString("switchMRID"));
                warnLine.setMaxI(rs.getDouble("maxI"));
                warnLine.setRatedCurrent(rs.getDouble("ratedCurrent"));
                warnLine.setLoadRate(rs.getDouble("loadRate"));
                warnLines.add(warnLine);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return warnLines;
    }

    /**
     * 查询按过载情况查询预警线路
     * @param tableName 表名
     * @return
     */
    public List<WarnLine> queryWarnLine(String tableName, int loadState) {
        List<WarnLine> warnLines = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where loadState=" + loadState;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                WarnLine warnLine = new WarnLine();
                warnLine.setLoadState(rs.getInt("loadState"));
                warnLine.setDevName(rs.getString("devName"));
                warnLine.setmRID(rs.getString("mRID"));
                warnLine.setSubstation(rs.getString("substation"));
                warnLine.setSwitchName(rs.getString("switchName"));
                warnLine.setSwitchMRID(rs.getString("switchMRID"));
                warnLine.setMaxI(rs.getDouble("maxI"));
                warnLine.setRatedCurrent(rs.getDouble("ratedCurrent"));
                warnLine.setLoadRate(rs.getDouble("loadRate"));
                warnLines.add(warnLine);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return warnLines;
    }

    /**
     * 查询预警公变
     * @param tableName 表名
     * @return
     */
    public List<WarnTf> queryWarnTf(String tableName) {
        List<WarnTf> warnTfs = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                WarnTf warnTf = new WarnTf();
                warnTf.setLoadState(rs.getInt("loadState"));
                warnTf.setDevName(rs.getString("devName"));
                warnTf.setmRID(rs.getString("mRID"));
                warnTf.setLineName(rs.getString("lineName"));
                warnTf.setLineMRID(rs.getString("lineMRID"));
                warnTf.setSubstation(rs.getString("substation"));
                warnTf.setMaxLoad(rs.getDouble("maxLoad"));
                warnTf.setRatedCap(rs.getDouble("ratedCap"));
                warnTf.setLoadRate(rs.getDouble("loadRate"));
                warnTfs.add(warnTf);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return warnTfs;
    }

    /**
     * 查询预警公变
     * @param tableName 表名
     * @return
     */
    public List<WarnTf> queryWarnTf(String tableName, int loadState) {
        List<WarnTf> warnTfs = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where loadState=" + loadState;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                WarnTf warnTf = new WarnTf();
                warnTf.setLoadState(rs.getInt("loadState"));
                warnTf.setDevName(rs.getString("devName"));
                warnTf.setmRID(rs.getString("mRID"));
                warnTf.setLineName(rs.getString("lineName"));
                warnTf.setLineMRID(rs.getString("lineMRID"));
                warnTf.setSubstation(rs.getString("substation"));
                warnTf.setMaxLoad(rs.getDouble("maxLoad"));
                warnTf.setRatedCap(rs.getDouble("ratedCap"));
                warnTf.setLoadRate(rs.getDouble("loadRate"));
                warnTfs.add(warnTf);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return warnTfs;
    }

    /**
     * 查询公变月三相不平衡度
     * @param tableName 表名
     * @return
     */
    public List<TfUb> queryTfMonthUb(String tableName) {
        List<TfUb> tfUbs = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                TfUb tfUb = new TfUb();
                tfUb.setDevName(rs.getString("devName"));
                tfUb.setmRID(rs.getString("mRID"));
                tfUb.setLineName(rs.getString("lineName"));
                tfUb.setLineMRID(rs.getString("lineMRID"));
                tfUb.setSubstation(rs.getString("substation"));
                tfUb.setMaxLoad(rs.getDouble("maxLoad"));
                tfUb.setRatedCap(rs.getDouble("ratedCap"));
                tfUb.setUb(rs.getDouble("ub"));
                double[] monthUb = new double[30];
                for (int i = 0; i < 30; i++) {
                    monthUb[i] = rs.getDouble("monthUb" + (i + 1));
                }
                tfUb.setMonthUb(monthUb);
                tfUbs.add(tfUb);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return tfUbs;
    }

    /**
     * 查询线路合格率
     * @param tableName 表名
     * @return
     */
    public List<LinePassRate> queryLinePassRate(String tableName) {
        List<LinePassRate> linePassRates = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                LinePassRate linePassRate = new LinePassRate();
                linePassRate.setFeederName(rs.getString("feederName"));
                linePassRate.setSubstation(rs.getString("substation"));
                double[] passRate = new double[4];
                passRate[0] = rs.getDouble("passRate1");
                passRate[1] = rs.getDouble("passRate2");
                passRate[2] = rs.getDouble("passRate3");
                passRate[3] = rs.getDouble("passRate4");
                linePassRate.setPassRate(passRate);
                linePassRates.add(linePassRate);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return linePassRates;
    }

    /**
     * 查询公变越限率
     * @param tableName 表名
     * @return
     */
    public List<TfOverLoadRate> queryTfOverLoadRate(String tableName) {
        List<TfOverLoadRate> tfOverLoadRates = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                TfOverLoadRate tfOverLoadRate = new TfOverLoadRate();
                tfOverLoadRate.setFeederName(rs.getString("feederName"));
                tfOverLoadRate.setDevName(rs.getString("devName"));
                tfOverLoadRate.setmRID(rs.getString("mRID"));
                tfOverLoadRate.setLineName(rs.getString("lineName"));
                tfOverLoadRate.setLineMRID(rs.getString("lineMRID"));
                tfOverLoadRate.setSubstation(rs.getString("substation"));
                double[] overLoadRate = new double[4];
                overLoadRate[0] = rs.getDouble("overLoadRate1");
                overLoadRate[1] = rs.getDouble("overLoadRate2");
                overLoadRate[2] = rs.getDouble("overLoadRate3");
                overLoadRate[3] = rs.getDouble("overLoadRate4");
                tfOverLoadRate.setOverLoadRate(overLoadRate);
                tfOverLoadRates.add(tfOverLoadRate);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return tfOverLoadRates;
    }

    /**
     * 按季节查询负荷接入分析结果总表
     * @param tableName 表名
     * @param season 季节
     * @return
     */
    public List<LoadPosSeason> queryLoadPosSeason(String tableName, int season) {
        List<LoadPosSeason> loadPosSeasons = new LinkedList<>();
        Connection conn = createConn();
        String sql;
        sql = "select * from " + tableName + " where season=" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                LoadPosSeason loadPosSeason = new LoadPosSeason();
                loadPosSeason.setLoadId(rs.getInt("loadId"));
                loadPosSeason.setSubstation(rs.getString("substation"));
                loadPosSeason.setFeederName(rs.getString("feederName"));
                loadPosSeason.setCanIn(rs.getString("canIn"));
                loadPosSeason.setTfName(rs.getString("tfName"));
                loadPosSeason.setTfMRID(rs.getString("tfMRID"));
                loadPosSeason.setPhase(rs.getString("phase"));
                loadPosSeason.setTime(rs.getString("time"));
                loadPosSeasons.add(loadPosSeason);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return loadPosSeasons;
    }

    /**
     * 查询负荷接入分析结果总表中最大负荷Id
     * @param tableName 表名
     * @return
     */
    public int queryMaxLoadId(String tableName) {
        int maxLoadId = 0;
        Connection conn = createConn();
        String sql;
        sql = "select max(loadId) as maxLoadId from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                maxLoadId = rs.getInt("maxLoadId");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return maxLoadId;
    }

    /**
     * 按loadId和季节查询负荷接入开关分析结果
     * @param tableName 表名
     * @param loadId
     * @param season 季节
     * @return
     */
    public LoadPosSw queryLoadPosSw(String tableName, int loadId, int season) {
        LoadPosSw loadPosSw = new LoadPosSw();
        Connection conn = createConn();
        String sql;
        sql = "select * from " + tableName + " where loadId='" + loadId + "' and season=" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                loadPosSw.setCanIn(rs.getString("canIn"));
                loadPosSw.setSwName(rs.getString("swName"));
                loadPosSw.setSwMRID(rs.getString("swMRID"));
                double[] newLoadI = new double[96];   // 接入的负荷电流
                double[] swOrgLoad = new double[96];   // 接入点开关原始负荷
                double[] swNewLoad = new double[96]; // 接入后开关电流
                double[] swRateI = new double[96]; // 接入点开关限额
                String[] newLoadIStr = rs.getString("newLoadI").split(";");
                String[] swOrgLoadStr = rs.getString("swOrgLoad").split(";");
                String[] swNewLoadStr = rs.getString("swNewLoad").split(";");
                double swRateIDouble = rs.getDouble("swRateI");
                for (int i = 0; i < 96; i++) {
                    newLoadI[i] = Double.parseDouble(newLoadIStr[i]);
                    swOrgLoad[i] = Double.parseDouble(swOrgLoadStr[i]);
                    swNewLoad[i] = Double.parseDouble(swNewLoadStr[i]);
                    swRateI[i] = swRateIDouble;
                }
                loadPosSw.setNewLoadI(newLoadI);
                loadPosSw.setSwOrgLoad(swOrgLoad);
                loadPosSw.setSwNewLoad(swNewLoad);
                loadPosSw.setSwRateI(swRateI);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return loadPosSw;
    }

    /**
     * 按loadId和季节查询负荷接入公变分析结果
     * @param tableName 表名
     * @param loadId
     * @param season 季节
     * @return
     */
    public LoadPosTf queryLoadPosTf(String tableName, int loadId, int season) {
        LoadPosTf loadPosTf = new LoadPosTf();
        Connection conn = createConn();
        String sql;
        sql = "select * from " + tableName + " where loadId='" + loadId + "' and season=" + season;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                loadPosTf.setCanIn(rs.getString("canIn"));
                loadPosTf.setTfName(rs.getString("tfName"));
                loadPosTf.setTfMRID(rs.getString("tfMRID"));
                loadPosTf.setPhase(rs.getString("phase"));
                double[] newLoad = new double[96];   // 接入的负荷电流
                double[] tfOrgLoad = new double[96];   // 接入点开关原始负荷
                double[] tfNewLoad = new double[96]; // 接入后开关电流
                double[] tfRateCap = new double[96]; // 接入点开关限额
                String[] newLoadStr = rs.getString("newLoad").split(";");
                String[] tfOrgLoadStr = rs.getString("tfOrgLoad").split(";");
                String[] tfNewLoadStr = rs.getString("tfNewLoad").split(";");
                double tfRateCapDouble = rs.getDouble("tfRateCap");
                for (int i = 0; i < 96; i++) {
                    newLoad[i] = Double.parseDouble(newLoadStr[i]);
                    tfOrgLoad[i] = Double.parseDouble(tfOrgLoadStr[i]);
                    tfNewLoad[i] = Double.parseDouble(tfNewLoadStr[i]);
                    tfRateCap[i] = tfRateCapDouble;
                }
                loadPosTf.setNewLoad(newLoad);
                loadPosTf.setTfOrgLoad(tfOrgLoad);
                loadPosTf.setTfNewLoad(tfNewLoad);
                loadPosTf.setTfRateCap(tfRateCap);
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return loadPosTf;
    }

    /**
     * 查询所有馈线名称
     * @param tableName 表名
     * @return
     */
    public List<String> queryAllFeederName(String tableName) {
        List<String> feederNames = new LinkedList<>();
        Connection conn = createConn();
        String sql = "select * from " + tableName;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                feederNames.add(rs.getString("feeder"));
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return feederNames;
    }

    /**
     * 按照过载类型和设备类型，查询预警设备数量
     * @param tableName 表名
     * @return
     */
    public int queryWarnDevNum(String tableName, int loadState, int devType) {
        int warnDevNum = 0;
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where loadState='" + loadState + "' and type=" + devType;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                warnDevNum = rs.getInt("number");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return warnDevNum;
    }

    /**
     * 查询可开放容量最大和最小两条线路
     * @param tableName 表名
     * @return
     */
    public MaxMinAvailCap queryMaxMinAvailCap(String tableName, int maxOrMin) {
        MaxMinAvailCap maxMinAvailCap = new MaxMinAvailCap();
        Connection conn = createConn();
        String sql = "select * from " + tableName + " where maxOrMin=" + maxOrMin;
        Statement stmt = null;
        ResultSet rs = null;
        try {
            stmt = conn.createStatement();
            rs = stmt.executeQuery(sql);
            while (rs.next()) {
                maxMinAvailCap.feederName = rs.getString("feederName");
                maxMinAvailCap.availCap = rs.getDouble("availCap");
            }
        } catch (SQLException e) {
            System.out.println(e.getMessage());
            e.printStackTrace();
        } finally {
            try {
                rs.close();
                stmt.close();
            } catch (SQLException e) {
            }
        }
        return maxMinAvailCap;
    }

    private Object createObj(String tableName, ResultSet rs) throws SQLException {
        return null;
    }
}
