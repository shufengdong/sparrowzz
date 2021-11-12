import com.alibaba.fastjson.JSON;
import com.alibaba.fastjson.JSONArray;
import com.alibaba.fastjson.JSONObject;
import zju.matrix.Complex;

import java.io.*;
import java.nio.charset.StandardCharsets;
import java.sql.Timestamp;
import java.text.DateFormat;
import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.*;

import static java.lang.Math.sqrt;

public class HistoryData {

    static String seasonTable = "_season";
    static String unbalanceTable = "_ub";
    static String minITable = "_minI";
    static String seasonClusterTable = "_seasonCluster";
    static String psClusterTable = "_psCluster";
    static String ubMaxTable = "_ubMax";
    static String tfAvailCapTable = "_availCap";

    static Map<String, Double> ratedIJK = new HashMap<String, Double>(){{
        put("35", 150.);
        put("50", 180.);
        put("70", 225.);
        put("95", 275.);
        put("120", 320.);
        put("150", 365.);
        put("185", 420.);
        put("240", 500.);
        put("300", 580.);
    }};
    static Map<String, Double> ratedILGJ = new HashMap<String, Double>(){{
        put("35", 170.);
        put("50", 220.);
        put("70", 275.);
        put("95", 335.);
        put("120", 380.);
        put("150", 445.);
        put("185", 515.);
        put("240", 610.);
        put("300", 754.);
    }};

    Complex a = new Complex(-1.0 / 2,sqrt(3) / 2);
    Complex a2 = Complex.multiply(a, a);
    Complex aThird = new Complex(1.0 / 3, 0);

    /**
     * 创建线路电流表格和按季度分析结果表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public static void createLineCurrentTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " date     datetime NULL," +
                " current              decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        initSql = "CREATE TABLE "  + tableName + seasonTable + " (" +
                " season     INTEGER NULL," +
                " current              decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + seasonTable;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建公变数据表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public static void createTFTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName     varchar(50) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " date     datetime NULL," +
                " Ia              decimal(8,4) NULL, " +
                " Ib              decimal(8,4) NULL, " +
                " Ic              decimal(8,4) NULL, " +
                " Ua              decimal(8,4) NULL, " +
                " Ub              decimal(8,4) NULL, " +
                " Uc              decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        initSql = "CREATE TABLE "  + tableName + seasonTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " P              decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 公变三相不平衡度分析结果表，season=-1为年平均不平衡度
        initSql = "CREATE TABLE "  + tableName + unbalanceTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " ubI2              decimal(2,7) NULL, " +
                " ubI0              decimal(2,7) NULL, " +
                " ubV2              decimal(2,7) NULL, " +
                " ubV0              decimal(2,7) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 公变最小相电流分析结果表，season=-1为年平均最小电流
        initSql = "CREATE TABLE "  + tableName + minITable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " phase              INTEGER NULL," +
                " minI              decimal(6,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 公变三相不平衡度最大值表
        initSql = "CREATE TABLE "  + tableName + ubMaxTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " ubI2              decimal(2,7) NULL, " +
                " ubV2              decimal(2,7) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 公变按季节聚类结果表
        initSql = "CREATE TABLE "  + tableName + seasonClusterTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " P              decimal(2,7) NULL, " +
                " ubI2             decimal(2,7) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 公变按峰谷聚类结果表
        initSql = "CREATE TABLE "  + tableName + psClusterTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " P              decimal(2,7) NULL, " +
                " ubI2             decimal(2,7) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 公变可开放容量结果表
        initSql = "CREATE TABLE "  + tableName + tfAvailCapTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " availCap             decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + seasonTable;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + unbalanceTable;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + minITable;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + ubMaxTable;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + seasonClusterTable;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + psClusterTable;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + tfAvailCapTable;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建公变月三相不平衡度表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public static void createTfMonthUbTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName     varchar(200) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " lineName     varchar(200) NOT NULL," +
                " lineMRID     varchar(50) NOT NULL," +
                " substation     varchar(20) NOT NULL," +
                " maxLoad        decimal(8,4) NULL," +
                " ratedCap       decimal(4,0) NULL," +
                " ub             decimal(2,7) NULL, " +
                " monthUb1        decimal(2,7) NULL, " +
                " monthUb2        decimal(2,7) NULL, " +
                " monthUb3        decimal(2,7) NULL, " +
                " monthUb4        decimal(2,7) NULL, " +
                " monthUb5        decimal(2,7) NULL, " +
                " monthUb6        decimal(2,7) NULL, " +
                " monthUb7        decimal(2,7) NULL, " +
                " monthUb8        decimal(2,7) NULL, " +
                " monthUb9        decimal(2,7) NULL, " +
                " monthUb10        decimal(2,7) NULL, " +
                " monthUb11        decimal(2,7) NULL, " +
                " monthUb12        decimal(2,7) NULL, " +
                " monthUb13        decimal(2,7) NULL, " +
                " monthUb14        decimal(2,7) NULL, " +
                " monthUb15        decimal(2,7) NULL, " +
                " monthUb16        decimal(2,7) NULL, " +
                " monthUb17        decimal(2,7) NULL, " +
                " monthUb18        decimal(2,7) NULL, " +
                " monthUb19        decimal(2,7) NULL, " +
                " monthUb20        decimal(2,7) NULL, " +
                " monthUb21        decimal(2,7) NULL, " +
                " monthUb22        decimal(2,7) NULL, " +
                " monthUb23        decimal(2,7) NULL, " +
                " monthUb24        decimal(2,7) NULL, " +
                " monthUb25        decimal(2,7) NULL, " +
                " monthUb26        decimal(2,7) NULL, " +
                " monthUb27        decimal(2,7) NULL, " +
                " monthUb28        decimal(2,7) NULL, " +
                " monthUb29        decimal(2,7) NULL, " +
                " monthUb30        decimal(2,7) NULL, " +
                " monthUb31        decimal(2,7) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建公变参数表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public static void createTFParamTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName     varchar(50) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " blLine     varchar(50) NOT NULL," +
                " ratedCap              decimal(4,0) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建智能开关数据表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public static void createSwitchTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName     varchar(50) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " date     datetime NULL," +
                " Ia              decimal(8,4) NULL, " +
                " Ib              decimal(8,4) NULL, " +
                " Ic              decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        initSql = "CREATE TABLE "  + tableName + seasonTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " Ia              decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        initSql = "CREATE TABLE "  + tableName + seasonClusterTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " Ia              decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        initSql = "CREATE TABLE "  + tableName + psClusterTable + " (" +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " Ia              decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + seasonTable;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + seasonClusterTable;
        sqliteDb.executeSql(sql);
        sql = "delete from " + tableName + psClusterTable;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建线路参数表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public static void createLineParamTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName     varchar(200) NOT NULL," +
                " pole1     varchar(100) NOT NULL," +
                " pole2     varchar(100) NOT NULL," +
                " ratedCurrent       decimal(6,2) NULL, " +
                " type     INTEGER NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建馈线名称表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createFeederNameTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " feeder     varchar(200) NOT NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 存储馈线名称
     */
    public void saveFeederName(String feederNameTable, String dbFile, String feederName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        List<String> sqls = new LinkedList<>();
        String insertSql = "insert into " + feederNameTable + " values('" + feederName + "')";
        sqls.add(insertSql);
        sqliteDb.executeSqls(sqls);
        sqls.clear();
    }

    /**
     * 创建线路合格率表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createLinePassRateTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " feederName     varchar(200) NOT NULL," +
                " substation     varchar(20) NOT NULL," +
                " passRate1       decimal(1,6) NULL," +
                " passRate2    decimal(1,6) NULL," +
                " passRate3      decimal(1,6) NULL," +
                " passRate4      decimal(1,6) NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建公变越限表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createTfOverLoadTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " feederName     varchar(200) NOT NULL," +
                " devName     varchar(200) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " lineName     varchar(200) NOT NULL," +
                " lineMRID     varchar(50) NOT NULL," +
                " substation     varchar(20) NOT NULL," +
                " overLoadRate1       decimal(1,6) NULL," +
                " overLoadRate2    decimal(1,6) NULL," +
                " overLoadRate3      decimal(1,6) NULL," +
                " overLoadRate4      decimal(1,6) NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 解析线路电流文件并存库
     * @param tableName 表名
     * @param filesPath 文件路径
     * @param dbFile 数据库路径
     */
    public static void parseLineCurrent(String tableName, String filesPath, String dbFile) {
        // 获取所有txt文件路径
        LinkedList<String> filePathList = new LinkedList<>();
        File[] fileList = new File(filesPath).listFiles();
        for(File file : fileList) {
            if(file.isFile()) {
                if (file.getName().endsWith(".txt")) {
                    filePathList.add(filesPath + "\\" + file.getName());
                }
            }
        }
        // 逐个解析文件
        for (String filePath : filePathList) {
            try {
                parseLineCurrent(tableName, new FileInputStream(new File(filePath)), dbFile);
            } catch (FileNotFoundException e) {
                e.printStackTrace();
            }
        }
    }

    public static void parseLineCurrent(String tableName, InputStream in, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        try {
            BufferedReader reader = new BufferedReader(new InputStreamReader(in, "GBK"));
            List<String> sqls = new LinkedList<>();
            String strLine = reader.readLine();
            String[] lastContent = new String[]{"0", "0"};
            while ((strLine = reader.readLine()) != null) {
                String[] contents = strLine.split("\t");
                if (contents.length == 1)
                    contents = new String[]{contents[0], lastContent[1]};
                // 处理datetime数据
                DateFormat simpleDateFormat= new SimpleDateFormat("yyyy-MM-dd HH:mm:ss"); //创建一个格式化日期对象
                Date date = null;
                try {
                    date = simpleDateFormat.parse(contents[0]);
                } catch (ParseException e) {
                    e.printStackTrace();
                }
                Timestamp timeStamp = new Timestamp(date.getTime()); // 让日期时间转换为数据库中的timestamp类型

                String insertSql = "insert into " + tableName + " values(" +
                        "'" + timeStamp + "'," + contents[1] + ")";
                sqls.add(insertSql);
                lastContent = contents;
            }
            sqliteDb.executeSqls(sqls);
            sqls.clear();
            reader.close();
            in.close();
        } catch (IOException e) {
            e.printStackTrace();
        }
    }

    /**
     * 解析公变参数文件并存库
     * @param tableName 表名
     * @param filePath 文件路径
     * @param dbFile 数据库路径
     */
    public static void parseTFParam(String tableName, String filePath, String dbFile) {
        try {
            InputStream in = new FileInputStream(new File(filePath));
            SqliteDb sqliteDb = new SqliteDb(dbFile);
            try {
                BufferedReader reader = new BufferedReader(new InputStreamReader(in, StandardCharsets.UTF_8));
                List<String> sqls = new LinkedList<>();
                String strLine = reader.readLine();
                reader.readLine();
                while ((strLine = reader.readLine()) != null) {
                    String[] contents = strLine.split(",");
                    double ratedCap = 400;
                    String insertSql = "insert into " + tableName + " values(" +
                            "'" + contents[1] + "','" + "0" + "','" + contents[3] + "'," + contents[5] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
                reader.close();
                in.close();
            } catch (IOException e) {
                e.printStackTrace();
            }
        } catch (FileNotFoundException e) {
            e.printStackTrace();
        }
    }

    /**
     * 解析公变数据文件并存库
     * @param tableName 表名
     * @param filesPath 文件路径
     * @param dbFile 数据库路径
     */
    public static void parseTF(String tableName, String filesPath, String dbFile) {
        // 获取所有txt文件路径
        LinkedList<String> filePathList = new LinkedList<>();
        File[] fileList = new File(filesPath).listFiles();
        for(File file : fileList) {
            if(file.isFile()) {
                if (file.getName().endsWith(".txt")) {
                    filePathList.add(filesPath + "\\" + file.getName());
                }
            }
        }
        // 逐个解析文件
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        for (String filePath : filePathList) {
            List<String> sqls = new LinkedList<>();
            String s = readJsonFile(filePath);
            JSONObject jobj = JSON.parseObject(s);
            String devName = (String)jobj.get("devName");
            JSONArray tfs = jobj.getJSONArray("root");   //构建JSONArray数组
            for (int i = 0; i < tfs.size(); i++) {
                JSONObject tf = (JSONObject) tfs.get(i);
                // 处理datetime数据
                DateFormat simpleDateFormat= new SimpleDateFormat("yyyy-MM-dd HH:mm:ss"); //创建一个格式化日期对象
                Date date = null;
                try {
                    date = simpleDateFormat.parse((String)tf.get("SJSJ"));
                } catch (ParseException e) {
                    e.printStackTrace();
                }
                Timestamp timeStamp = new Timestamp(date.getTime()); // 让日期时间转换为数据库中的timestamp类型

                String insertSql = "insert into " + tableName + " values(" +
                        "'" + devName + "','" + (((String)tf.get("zybh")).replace('-','_')) + "','" +
                        timeStamp + "'," + tf.get("Ia") + "," + tf.get("Ib") + "," + tf.get("Ic") + "," +
                        tf.get("Ua") + "," + tf.get("Ub") + "," + tf.get("Uc") + ")";
                sqls.add(insertSql);
            }
            sqliteDb.executeSqls(sqls);
            sqls.clear();
        }
    }

    /**
     * 解析公变参数文件并存库
     * @param tfParamTableName 公变参数表名
     * @param tfTableName 公变表名
     * @param dbFile 数据库路径
     */
    public static void parseTFMRID(String tfParamTableName, String tfTableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        Map<String, String> nameToMRID = sqliteDb.queryNameToMRID(tfTableName);
        List<String> sqls = new LinkedList<>();
        for (String name : nameToMRID.keySet()) {
            String updateSql = "update " + tfParamTableName + " set mRID = '" + nameToMRID.get(name) + "' where devName = '" + name + "'";
            sqls.add(updateSql);
        }
        sqliteDb.executeSqls(sqls);
        sqls.clear();
    }

    /**
     * 解析开关数据文件并存库
     * @param tableName 表名
     * @param filesPath 文件路径
     * @param dbFile 数据库路径
     */
    public static void parseSwitch(String tableName, String filesPath, String dbFile) {
        // 获取所有txt文件路径
        LinkedList<String> filePathList = new LinkedList<>();
        File[] fileList = new File(filesPath).listFiles();
        for(File file : fileList) {
            if(file.isFile()) {
                if (file.getName().endsWith(".txt")) {
                    filePathList.add(filesPath + "\\" + file.getName());
                }
            }
        }
        // 逐个解析文件
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        for (String filePath : filePathList) {
            List<String> sqls = new LinkedList<>();
            String s = readJsonFile(filePath);
            JSONObject jobj = JSON.parseObject(s);
            String devName = (String)jobj.get("devName");
            JSONArray sws = jobj.getJSONArray("root");   //构建JSONArray数组
            for (int i = 0; i < sws.size(); i++) {
                JSONObject sw = (JSONObject) sws.get(i);
                // 处理datetime数据
                DateFormat simpleDateFormat= new SimpleDateFormat("yyyy-MM-dd HH:mm:ss"); //创建一个格式化日期对象
                Date date = null;
                try {
                    date = simpleDateFormat.parse((String)sw.get("SJSJ"));
                } catch (ParseException e) {
                    e.printStackTrace();
                }
                Timestamp timeStamp = new Timestamp(date.getTime()); // 让日期时间转换为数据库中的timestamp类型

                String insertSql = "insert into " + tableName + " values(" +
                        "'" + devName + "','" + (((String)sw.get("zybh")).replace('-','_')) + "','" +
                        timeStamp + "'," + sw.get("Iax") + "," + sw.get("Ibx") + "," + sw.get("Icx") + ")";
                sqls.add(insertSql);
            }
            sqliteDb.executeSqls(sqls);
            sqls.clear();
        }
    }

    //读取json文件
    public static String readJsonFile(String fileName) {
        String jsonStr = "";
        try {
            File jsonFile = new File(fileName);
            FileReader fileReader = new FileReader(jsonFile);
            Reader reader = new InputStreamReader(new FileInputStream(jsonFile),"utf-8");
            int ch = 0;
            StringBuffer sb = new StringBuffer();
            while ((ch = reader.read()) != -1) {
                sb.append((char) ch);
            }
            fileReader.close();
            reader.close();
            jsonStr = sb.toString();
            return jsonStr;
        } catch (IOException e) {
            e.printStackTrace();
            return null;
        }
    }

    /**
     * 解析线路参数文件并存库
     * @param tableName 表名
     * @param filePath 文件路径
     * @param dbFile 数据库路径
     */
    public static void parseLineParam(String tableName, String filePath, String dbFile) {
        try {
            InputStream in = new FileInputStream(new File(filePath));
            SqliteDb sqliteDb = new SqliteDb(dbFile);
            try {
                BufferedReader reader = new BufferedReader(new InputStreamReader(in, StandardCharsets.UTF_8));
                List<String> sqls = new LinkedList<>();
                String strLine = reader.readLine();
                while ((strLine = reader.readLine()) != null) {
                    String[] contents = strLine.split(",");
                    double ratedCurrent = 1000;
                    String[] type = contents[6].split("-");
                    int tp = 1;
                    if (type[0].startsWith("JKLYJ") || type[0].equals("JKLGYJ") || type[0].equals("JLYJ") || type[0].equals("JKYJ")) {
                        if (type.length == 3) {
                            if (ratedIJK.get(type[2]) != null) {
                                ratedCurrent = ratedIJK.get(type[2]);
                            }
                        } else if (type.length == 2) {
                            if (type[1].split("/").length > 1) {
                                if (ratedIJK.get(type[1].split("/")[1]) != null) {
                                    ratedCurrent = ratedIJK.get(type[1].split("/")[1]);
                                }
                            } else {
                                if (ratedIJK.get(type[1]) != null) {
                                    ratedCurrent = ratedIJK.get(type[1]);
                                }
                            }
                        }
                    } else if (type[0].startsWith("JL/")) {
                        if (type.length > 1) {
                            if (ratedIJK.get(type[1].split("/")[0]) != null) {
                                ratedCurrent = ratedIJK.get(type[1].split("/")[0]);
                            }
                        }
                    } else if (type[0].equals("LGJ")) {
                        tp = 2;
                        if (type.length > 1) {
                            if (ratedILGJ.get(type[1].split("/")[0]) != null) {
                                ratedCurrent = ratedILGJ.get(type[1].split("/")[0]);
                            }
                        }
                    } else {
                        System.out.println(contents[6]);
                    }
                    String insertSql = "insert into " + tableName + " values(" +
                            "'" + contents[0] + "','" + contents[7] + "','" + contents[8] + "'," + ratedCurrent + "," + tp + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
                reader.close();
                in.close();
            } catch (IOException e) {
                e.printStackTrace();
            }
        } catch (FileNotFoundException e) {
            e.printStackTrace();
        }
    }

    /**
     * 按季节处理线电流历史数据，每个时段电流取该季节的最大值
     * @param tableName 表名
     * @param dbFile 数据库路径
     */
    public void lineCurrentDataAnalysis(String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        int T = 5; // 采点时间间隔
        int pointNum = 24 * 60 / T;
        for (int season = 1; season < 5; season++) {
            double[] maxCurrent = new double[pointNum];
            List<LineCurrentData> lineCurrentDatas = sqliteDb.queryLineCurrentData(tableName, season);
            Iterator<LineCurrentData> iterator = lineCurrentDatas.iterator();
            while (iterator.hasNext()) {
                LineCurrentData lineCurrentData = iterator.next();
                long time = lineCurrentData.getData().getTime();
                Calendar calendar = Calendar.getInstance();
                calendar.setTimeInMillis(time);
                int hour = calendar.get(Calendar.HOUR_OF_DAY);
                int minutes = calendar.get(Calendar.MINUTE);
                int tNum = (60 * hour + minutes) / T;
                if (maxCurrent[tNum] < lineCurrentData.getCurrent())
                    maxCurrent[tNum] = lineCurrentData.getCurrent();
            }
            List<String> sqls = new LinkedList<>();
            for (int i = 0; i < maxCurrent.length; i++) {
                String insertSql = "insert into " + tableName + seasonTable + " values(" +
                        season + "," + maxCurrent[i] + ")";
                sqls.add(insertSql);
            }
            sqliteDb.executeSqls(sqls);
            sqls.clear();
        }
    }

    /**
     * 按季节处理开关历史数据，每个时段电流取该季节的最大值
     * @param tableName 表名
     * @param dbFile 数据库路径
     */
    public void switchDataAnalysis(String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        int T = 15; // 采点时间间隔
        int pointNum = 24 * 60 / T;
        List<String> mRIDs = sqliteDb.queryMRIds(tableName);
        for (int i = 0; i < mRIDs.size(); i++) {
            double yMaxI = 0;
            double avgI = 0;
            int count = 0;
            for (int season = 1; season < 5; season++) {
                double[] maxCurrent = new double[pointNum];
                List<SwitchData> swDatas = sqliteDb.querySwitchData(tableName, mRIDs.get(i), season);
                Iterator<SwitchData> iterator = swDatas.iterator();
                while (iterator.hasNext()) {
                    SwitchData swData = iterator.next();
                    long time = swData.getData().getTime();
                    Calendar calendar = Calendar.getInstance();
                    calendar.setTimeInMillis(time);
                    int hour = calendar.get(Calendar.HOUR_OF_DAY);
                    int minutes = calendar.get(Calendar.MINUTE);
                    int tNum = (60 * hour + minutes) / T;
                    if (maxCurrent[tNum] < swData.getIa())
                        maxCurrent[tNum] = swData.getIa();
                    if (yMaxI < swData.getIa())
                        yMaxI = swData.getIa();
                    avgI += swData.getIa();
                    count++;
                }
                List<String> sqls = new LinkedList<>();
                for (int j = 0; j < pointNum; j++) {
                    String insertSql = "insert into " + tableName + seasonTable + " values(" +
                            "'" + mRIDs.get(i) + "'," + season + "," + maxCurrent[j] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
            List<String> sqls = new LinkedList<>();
            String insertSql = "insert into " + tableName + seasonTable + " values(" +
                    "'" + mRIDs.get(i) + "'," + (-1) + "," + yMaxI + ")";
            sqls.add(insertSql);
            avgI /= count;
            insertSql = "insert into " + tableName + seasonTable + " values(" +
                    "'" + mRIDs.get(i) + "'," + (-2) + "," + avgI + ")";
            sqls.add(insertSql);
            sqliteDb.executeSqls(sqls);
            sqls.clear();
        }
    }

    /**
     * 按季节聚类开关历史数据
     * @param tableName 表名
     * @param dbFile 数据库路径
     */
    public void switchDataSeasonCluster(String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        int T = 15; // 采点时间间隔
        int pointNum = 24 * 60 / T;
        List<String> mRIDs = sqliteDb.queryMRIds(tableName);
        for (int i = 0; i < mRIDs.size(); i++) {
            for (int season = 1; season < 5; season++) {
                double[] center = new double[pointNum];
                int[] count = new int[pointNum];
                List<SwitchData> swDatas = sqliteDb.querySwitchData(tableName, mRIDs.get(i), season);
                Iterator<SwitchData> iterator = swDatas.iterator();
                while (iterator.hasNext()) {
                    SwitchData swData = iterator.next();
                    long time = swData.getData().getTime();
                    Calendar calendar = Calendar.getInstance();
                    calendar.setTimeInMillis(time);
                    int hour = calendar.get(Calendar.HOUR_OF_DAY);
                    int minutes = calendar.get(Calendar.MINUTE);
                    int tNum = (60 * hour + minutes) / T;
                    center[tNum] += swData.getIa();
                    count[tNum]++;
                }
                for (int j = 0; j < pointNum; j++) {
                    if (count[j] > 0)
                        center[j] /= count[j];
                }
                List<String> sqls = new LinkedList<>();
                for (int j = 0; j < pointNum; j++) {
                    String insertSql = "insert into " + tableName + seasonClusterTable + " values(" +
                            "'" + mRIDs.get(i) + "'," + season + "," + center[j] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
        }
    }

    /**
     * 按峰谷聚类开关历史数据
     * @param tableName 表名
     * @param dbFile 数据库路径
     */
    public void switchDataPsCluster(String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        int T = 15; // 采点时间间隔
        int pointNum = 24 * 60 / T;
        List<String> mRIDs = sqliteDb.queryMRIds(tableName);
        for (int i = 0; i < mRIDs.size(); i++) {
            for (int season = 1; season < 5; season++) {
                double[] center = new double[pointNum];
                int[] count = new int[pointNum];
                for (int month = season * 3 - 2; month < season * 3 + 1; month++) {
                    List<SwitchData> swDatas = sqliteDb.querySwDataByMonth(tableName, mRIDs.get(i), month);
                    Iterator<SwitchData> iterator = swDatas.iterator();
                    while (iterator.hasNext()) {
                        SwitchData swData = iterator.next();
                        long time = swData.getData().getTime();
                        Calendar calendar = Calendar.getInstance();
                        calendar.setTimeInMillis(time);
                        int hour = calendar.get(Calendar.HOUR_OF_DAY);
                        int minutes = calendar.get(Calendar.MINUTE);
                        int tNum = (60 * hour + minutes) / T;
                        center[tNum] += swData.getIa();
                        count[tNum]++;
                    }
                    boolean hasData = true;
                    for (int j = 0; j < pointNum; j++) {
                        if (count[j] == 0)
                            hasData = false;
                    }
                    if (hasData)
                        break;
                }
                for (int j = 0; j < pointNum; j++) {
                    if (count[j] > 0)
                        center[j] /= count[j];
                }
                List<String> sqls = new LinkedList<>();
                for (int j = 0; j < pointNum; j++) {
                    String insertSql = "insert into " + tableName + psClusterTable + " values(" +
                            "'" + mRIDs.get(i) + "'," + season + "," + center[j] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
        }
    }

    /**
     * 按季节处理公变历史数据，每个时段负荷取该季节的最大值，按季节和年计算平均三相不平衡度
     * @param tableName 表名
     * @param dbFile 数据库路径
     */
    public void tfDataAnalysis(String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        int T = 15; // 采点时间间隔
        int pointNum = 24 * 60 / T;
        List<String> mRIDs = sqliteDb.queryMRIds(tableName);
        for (int i = 0; i < mRIDs.size(); i++) {
            double[][] mAvgUbI = new double[4][2];   // 季度平均三相电流不平衡度
            double[][] mAvgUbV = new double[4][2];   // 季度平均三相电压不平衡度
            double[][] mAvgI = new double[4][3];   // 季度平均三相电流
            int[] mMinIPhase = new int[4];
            double[] mMinI = new double[4];
            double maxI = 0;
            double avgI = 0;
            int count = 0;
            int seasonNum = 4;  // 存在季度数据空缺的情况
            double yMaxP = 0;
            for (int season = 1; season < 5; season++) {
                double[] maxP = new double[pointNum];   // 公变最大负荷
                double[] maxUbI = new double[pointNum];   // 公变电流最大三相不平衡度
                double[] maxUbV = new double[pointNum];   // 公变电压最大三相不平衡度
                List<TFData> tfDatas = sqliteDb.queryTFData(tableName, mRIDs.get(i), season);
                Iterator<TFData> iterator = tfDatas.iterator();
                while (iterator.hasNext()) {
                    TFData tfData = iterator.next();
                    long time = tfData.getDate().getTime();
                    Calendar calendar = Calendar.getInstance();
                    calendar.setTimeInMillis(time);
                    int hour = calendar.get(Calendar.HOUR_OF_DAY);
                    int minutes = calendar.get(Calendar.MINUTE);
                    int tNum = (60 * hour + minutes) / T;
                    double P = tfData.getIa() * tfData.getUa() + tfData.getIb() * tfData.getUb() + tfData.getIc() * tfData.getUc();
                    if (maxP[tNum] < P)
                        maxP[tNum] = P;
                    if (yMaxP < P)
                        yMaxP = P;
                    if (maxI < tfData.getIa())
                        maxI = tfData.getIa();
                    if (maxI < tfData.getIb())
                        maxI = tfData.getIb();
                    if (maxI < tfData.getIc())
                        maxI = tfData.getIc();
                    avgI += tfData.getIa() + tfData.getIb() + tfData.getIc();
                    count += 3;
                    // 计算三相不平衡度
                    double[] ubI = unbalance(new Complex(tfData.getIa(), 0),
                            Complex.multiply(new Complex(tfData.getIb(), 0), a2),
                            Complex.multiply(new Complex(tfData.getIc(), 0), a));
                    mAvgUbI[season - 1][0] += ubI[0];
                    mAvgUbI[season - 1][1] += ubI[1];
                    double[] ubV = unbalance(new Complex(tfData.getUa(), 0),
                            Complex.multiply(new Complex(tfData.getUb(), 0), a2),
                            Complex.multiply(new Complex(tfData.getUc(), 0), a));
                    mAvgUbV[season - 1][0] += ubV[0];
                    mAvgUbV[season - 1][1] += ubV[1];
                    // 更新三相不平衡度最大值
                    if (maxUbI[tNum] < ubI[0])
                        maxUbI[tNum] = ubI[0];
                    if (maxUbV[tNum] < ubV[0])
                        maxUbV[tNum] = ubV[0];
                    // 计算季度平均三相电流
                    mAvgI[season - 1][0] += tfData.getIa();
                    mAvgI[season - 1][1] += tfData.getIb();
                    mAvgI[season - 1][2] += tfData.getIc();
                }
                if (tfDatas.size() == 0) {
                    seasonNum--;
                } else {
                    mAvgUbI[season - 1][0] /= tfDatas.size();
                    mAvgUbI[season - 1][1] /= tfDatas.size();
                    mAvgUbV[season - 1][0] /= tfDatas.size();
                    mAvgUbV[season - 1][1] /= tfDatas.size();
                    mAvgI[season - 1][0] /= tfDatas.size();
                    mAvgI[season - 1][1] /= tfDatas.size();
                    mAvgI[season - 1][2] /= tfDatas.size();
                    // 分析季度平均电流最小相
                    mMinI[season - 1] = mAvgI[season - 1][0];
                    mMinIPhase[season - 1] = 0;
                    if (mMinI[season - 1] > mAvgI[season - 1][1]) {
                        mMinI[season - 1] = mAvgI[season - 1][1];
                        mMinIPhase[season - 1] = 1;
                    }
                    if (mMinI[season - 1] > mAvgI[season - 1][2]) {
                        mMinI[season - 1] = mAvgI[season - 1][2];
                        mMinIPhase[season - 1] = 2;
                    }
                }
                // 公变负荷数据存库
                List<String> sqls = new LinkedList<>();
                for (int j = 0; j < pointNum; j++) {
                    String insertSql = "insert into " + tableName + seasonTable + " values(" +
                            "'" + mRIDs.get(i) + "'," + season + "," + maxP[j] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
                // 最大三相不平衡度存库
                for (int j = 0; j < pointNum; j++) {
                    String insertSql = "insert into " + tableName + ubMaxTable + " values(" +
                            "'" + mRIDs.get(i) + "'," + season + "," + maxUbI[j] + "," + maxUbV[j] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
            // 公变年最大负荷存库
            List<String> sqls = new LinkedList<>();
            String insertSql = "insert into " + tableName + seasonTable + " values(" +
                    "'" + mRIDs.get(i) + "'," + (-1) + "," + yMaxP + ")";
            sqls.add(insertSql);
            sqliteDb.executeSqls(sqls);
            sqls.clear();
            // 公变最大电流存库
            insertSql = "insert into " + tableName + seasonTable + " values(" +
                    "'" + mRIDs.get(i) + "'," + (-2) + "," + maxI + ")";
            sqls.add(insertSql);
            sqliteDb.executeSqls(sqls);
            sqls.clear();
            // 公变平均电流存库
            avgI /= count;
            insertSql = "insert into " + tableName + seasonTable + " values(" +
                    "'" + mRIDs.get(i) + "'," + (-3) + "," + avgI + ")";
            sqls.add(insertSql);
            sqliteDb.executeSqls(sqls);
            sqls.clear();
            // 年平均三相不平衡度
            double[] yAvgUbI = new double[2];
            double[] yAvgUbV = new double[2];
            for (int season = 0; season < 4; season++) {
                yAvgUbI[0] += mAvgUbI[season][0];
                yAvgUbI[1] += mAvgUbI[season][1];
                yAvgUbV[0] += mAvgUbV[season][0];
                yAvgUbV[1] += mAvgUbV[season][1];
            }
            yAvgUbI[0] /= seasonNum;
            yAvgUbI[1] /= seasonNum;
            yAvgUbV[0] /= seasonNum;
            yAvgUbV[1] /= seasonNum;
            // 三相不平衡度分析结果存库
            sqls = new LinkedList<>();
            for (int season = 1; season < 5; season++) {
                insertSql = "insert into " + tableName + unbalanceTable + " values(" +
                        "'" + mRIDs.get(i) + "'," + season + "," + mAvgUbI[season - 1][0] + "," + mAvgUbI[season - 1][1] +
                        "," + mAvgUbV[season - 1][0] + "," + mAvgUbV[season - 1][1] + ")";
                sqls.add(insertSql);
            }
            insertSql = "insert into " + tableName + unbalanceTable + " values(" +
                    "'" + mRIDs.get(i) + "'," + (-1) + "," + yAvgUbI[0] + "," + yAvgUbI[1] + "," + yAvgUbV[0] + "," + yAvgUbV[1] + ")";
            sqls.add(insertSql);
            sqliteDb.executeSqls(sqls);
            sqls.clear();
            // 计算年平均三相电流
            double[] yAvgI = new double[3];
            for (int season = 0; season < 4; season++) {
                yAvgI[0] += mAvgI[season][0];
                yAvgI[1] += mAvgI[season][1];
                yAvgI[2] += mAvgI[season][2];
            }
            yAvgI[0] /= seasonNum;
            yAvgI[1] /= seasonNum;
            yAvgI[2] /= seasonNum;
            // 分析年平均电流最小相
            int yMinIPhase = 0;
            double yMinI = yAvgI[0];
            if (yMinI > yAvgI[1]) {
                yMinI = yAvgI[1];
                yMinIPhase = 1;
            }
            if (yMinI > yAvgI[2]) {
                yMinI = yAvgI[2];
                yMinIPhase = 2;
            }
            // 平均最小相电流分析结果存库
            for (int season = 1; season < 5; season++) {
                insertSql = "insert into " + tableName + minITable + " values(" +
                        "'" + mRIDs.get(i) + "'," + season + "," + mMinIPhase[season - 1] + "," + mMinI[season - 1] + ")";
                sqls.add(insertSql);
            }
            insertSql = "insert into " + tableName + minITable + " values(" +
                    "'" + mRIDs.get(i) + "'," + (-1) + "," + yMinIPhase + "," + yMinI + ")";
            sqls.add(insertSql);
            sqliteDb.executeSqls(sqls);
            sqls.clear();
        }
    }

    /**
     * 公变月三相不平衡度
     * @param tfTable 表名
     * @param dbFile 数据库路径
     */
    public void tfMonthUb(String tfTable, String dbFile, String tfMonthUbTable, String substationTable, String tfParamTable,
                          String tfSeasonTable, String tfToLineTable) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        // 查询变电站名称
        String substation = sqliteDb.querySubstationName(substationTable);
        Map<String, String> tfNameToMRID = sqliteDb.queryNameToMRID(tfTable);
        for (String name : tfNameToMRID.keySet()) {
            String mRID = tfNameToMRID.get(name);
            double tfRatedCap = sqliteDb.queryTFCap(tfParamTable, mRID);
            double tfMaxP = sqliteDb.queryMaxTFP(tfSeasonTable, mRID, -1) / 1000;
            double ub = 0;  // 年平均三相不平衡度
            int ubCount = 0;
            double[] monthUb = new double[31];  // 月三相不平衡度
            int[] count = new int[31];
            for (int season = 1; season < 5; season++) {
                List<TFData> tfDatas = sqliteDb.queryTFData(tfTable, mRID, season);
                Iterator<TFData> iterator = tfDatas.iterator();
                while (iterator.hasNext()) {
                    TFData tfData = iterator.next();
                    long time = tfData.getDate().getTime();
                    Calendar calendar = Calendar.getInstance();
                    calendar.setTimeInMillis(time);
                    int day = calendar.get(Calendar.DAY_OF_MONTH);
                    // 计算三相不平衡度
                    double[] ubI = unbalance(new Complex(tfData.getIa(), 0),
                            Complex.multiply(new Complex(tfData.getIb(), 0), a2),
                            Complex.multiply(new Complex(tfData.getIc(), 0), a));
                    ub += ubI[0];
                    ubCount++;
                    monthUb[day - 1] += ubI[0];
                    count[day - 1]++;
                }
            }
            ub /= ubCount;
            for (int j = 0; j < 31; j++) {
                if (count[j] > 0) {
                    monthUb[j] /= count[j];
                }
            }
            // 公变对应的线路名称
            List<String[]> lines = sqliteDb.querySwToDev(tfToLineTable, mRID);
            if (lines.size() > 0) {
                String[] line = lines.get(0);
                // 公变月三相不平衡度存库
                List<String> sqls = new LinkedList<>();
                String insertSql = "insert into " + tfMonthUbTable + " values(" +
                        "'" + name + "','" + mRID + "','" + line[0] + "','" + line[1] + "','" + substation + "'," +
                        tfMaxP + "," + tfRatedCap + "," + ub + "," + monthUb[0] + "," + monthUb[1] + "," + monthUb[2] + "," +
                        monthUb[3] + "," + monthUb[4] + "," + monthUb[5] + "," + monthUb[6] + "," + monthUb[7] + "," +
                        monthUb[8] + "," + monthUb[9] + "," + monthUb[10] + "," + monthUb[11] + "," + monthUb[12] + "," +
                        monthUb[13] + "," + monthUb[14] + "," + monthUb[15] + "," + monthUb[16] + "," + monthUb[17] + "," +
                        monthUb[18] + "," + monthUb[19] + "," + monthUb[20] + "," + monthUb[21] + "," + monthUb[22] + "," +
                        monthUb[23] + "," + monthUb[24] + "," + monthUb[25] + "," + monthUb[26] + "," + monthUb[27] + "," +
                        monthUb[28] + "," + monthUb[29] + "," + monthUb[30] + ")";
                sqls.add(insertSql);
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
        }
    }

    /**
     * 按季节聚类公变负荷历史数据和三相最大不平衡度
     * @param tableName 表名
     * @param dbFile 数据库路径
     */
    public void tfDataSeasonCluster(String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        int T = 15; // 采点时间间隔
        int pointNum = 24 * 60 / T;
        List<String> mRIDs = sqliteDb.queryMRIds(tableName);
        for (int i = 0; i < mRIDs.size(); i++) {
            int seasonNum = 4;  // 存在季度数据空缺的情况
            for (int season = 1; season < 5; season++) {
                double[] centerP = new double[pointNum];    // 负荷聚类中心
                double[] centerUb = new double[pointNum];    // 不平衡度聚类中心
                int[] count = new int[pointNum];
                List<TFData> tfDatas = sqliteDb.queryTFData(tableName, mRIDs.get(i), season);
                Iterator<TFData> iterator = tfDatas.iterator();
                while (iterator.hasNext()) {
                    TFData tfData = iterator.next();
                    long time = tfData.getDate().getTime();
                    Calendar calendar = Calendar.getInstance();
                    calendar.setTimeInMillis(time);
                    int hour = calendar.get(Calendar.HOUR_OF_DAY);
                    int minutes = calendar.get(Calendar.MINUTE);
                    int tNum = (60 * hour + minutes) / T;
                    double P = tfData.getIa() * tfData.getUa() + tfData.getIb() * tfData.getUb() + tfData.getIc() * tfData.getUc();
                    centerP[tNum] += P;
                    // 计算三相不平衡度
                    double[] ubI = unbalance(new Complex(tfData.getIa(), 0),
                            Complex.multiply(new Complex(tfData.getIb(), 0), a2),
                            Complex.multiply(new Complex(tfData.getIc(), 0), a));
                    centerUb[tNum] += ubI[0];
                    count[tNum]++;
                }
                for (int j = 0; j < pointNum; j++) {
                    if (count[j] > 0) {
                        centerP[j] /= count[j];
                        centerUb[j] /= count[j];
                    }
                }
                // 公变按季节聚类结果存库
                List<String> sqls = new LinkedList<>();
                for (int j = 0; j < pointNum; j++) {
                    String insertSql = "insert into " + tableName + seasonClusterTable + " values(" +
                            "'" + mRIDs.get(i) + "'," + season + "," + centerP[j] + "," + centerUb[j] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
        }
    }

    /**
     * 按峰谷聚类公变负荷历史数据和三相最大不平衡度
     * @param tableName 表名
     * @param dbFile 数据库路径
     */
    public void tfDataPsCluster(String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        int T = 15; // 采点时间间隔
        int pointNum = 24 * 60 / T;
        List<String> mRIDs = sqliteDb.queryMRIds(tableName);
        for (int i = 0; i < mRIDs.size(); i++) {
            int seasonNum = 4;  // 存在季度数据空缺的情况
            for (int season = 1; season < 5; season++) {
                double[] centerP = new double[pointNum];    // 负荷聚类中心
                double[] centerUb = new double[pointNum];    // 不平衡度聚类中心
                int[] count = new int[pointNum];
                for (int month = season * 3 - 2; month < season * 3 + 1; month++) {
                    List<TFData> tfDatas = sqliteDb.queryTFDataByMonth(tableName, mRIDs.get(i), month);
                    Iterator<TFData> iterator = tfDatas.iterator();
                    while (iterator.hasNext()) {
                        TFData tfData = iterator.next();
                        long time = tfData.getDate().getTime();
                        Calendar calendar = Calendar.getInstance();
                        calendar.setTimeInMillis(time);
                        int hour = calendar.get(Calendar.HOUR_OF_DAY);
                        int minutes = calendar.get(Calendar.MINUTE);
                        int tNum = (60 * hour + minutes) / T;
                        double P = tfData.getIa() * tfData.getUa() + tfData.getIb() * tfData.getUb() + tfData.getIc() * tfData.getUc();
                        centerP[tNum] += P;
                        // 计算三相不平衡度
                        double[] ubI = unbalance(new Complex(tfData.getIa(), 0),
                                Complex.multiply(new Complex(tfData.getIb(), 0), a2),
                                Complex.multiply(new Complex(tfData.getIc(), 0), a));
                        centerUb[tNum] += ubI[0];
                        count[tNum]++;
                    }
                    boolean hasData = true;
                    for (int j = 0; j < pointNum; j++) {
                        if (count[j] == 0)
                            hasData = false;
                    }
                    if (hasData)
                        break;
                }
                for (int j = 0; j < pointNum; j++) {
                    if (count[j] > 0) {
                        centerP[j] /= count[j];
                        centerUb[j] /= count[j];
                    }
                }
                // 公变按季节聚类结果存库
                List<String> sqls = new LinkedList<>();
                for (int j = 0; j < pointNum; j++) {
                    String insertSql = "insert into " + tableName + psClusterTable + " values(" +
                            "'" + mRIDs.get(i) + "'," + season + "," + centerP[j] + "," + centerUb[j] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
        }
    }

    /**
     * 按季节计算公变可开放容量
     * @param tableName 表名
     * @param dbFile 数据库路径
     */
    public void tfAvailCap(String tfSeasonTable, String tfParamTable, String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        int T = 15; // 采点时间间隔
        int pointNum = 24 * 60 / T;
        List<String> mRIDs = sqliteDb.queryMRIds(tfSeasonTable);
        for (int i = 0; i < mRIDs.size(); i++) {
            double tFRatedCap = sqliteDb.queryTFCap(tfParamTable, mRIDs.get(i));
            for (int season = 1; season < 5; season++) {
                double[] seasonP = sqliteDb.querySeasonTFP(tfSeasonTable, mRIDs.get(i), season, 96);
                double[] availCap = new double[pointNum];
                for (int j = 0; j < pointNum; j++) {
                    availCap[j] = tFRatedCap - seasonP[j] / 1000;
                }
                // 公变可接入容量存库
                List<String> sqls = new LinkedList<>();
                for (int j = 0; j < pointNum; j++) {
                    String insertSql = "insert into " + tableName + tfAvailCapTable + " values(" +
                            "'" + mRIDs.get(i) + "'," + season + "," + availCap[j] + ")";
                    sqls.add(insertSql);
                }
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
        }
    }

    /**
     * 公变越限分析
     */
    public void tfOverLoad(String dbFile, String feederTable, String substationTable, String tfOverLoadTable, String tfTable, String tfParamTable,
                           String tfSeasonTable, String tfToLineTable) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        // 查询馈线名称
        String feeder = sqliteDb.queryFeederName(feederTable);
        // 查询变电站名称
        String substation = sqliteDb.querySubstationName(substationTable);
        // 分析公变
        Map<String, String> tfNameToMRID = sqliteDb.queryNameToMRID(tfTable);
        List<String> sqls = new LinkedList<>();
        for (String name : tfNameToMRID.keySet()) {
            String mRID = tfNameToMRID.get(name);
            double tfRatedCap = sqliteDb.queryTFCap(tfParamTable, mRID);
            // 公变对应的线路名称
            List<String[]> lines = sqliteDb.querySwToDev(tfToLineTable, mRID);
            if (lines.size() > 0) {
                String[] line = lines.get(0);
                double[] overLoadRate = new double[4];
                for (int season = 1; season < 5; season++) {
                    int count = 0;
                    List<TFData> tfDatas = sqliteDb.queryTFData(tfTable, mRID, season);
                    Iterator<TFData> iterator = tfDatas.iterator();
                    while (iterator.hasNext()) {
                        TFData tfData = iterator.next();
                        double P = tfData.getIa() * tfData.getUa() + tfData.getIb() * tfData.getUb() + tfData.getIc() * tfData.getUc();
                        double loadRate = P / tfRatedCap;
                        if (loadRate < 2 && loadRate > 1) {
                            overLoadRate[season - 1]++;
                        }
                        count++;
                    }
                    overLoadRate[season - 1] /= count;
                }
                String insertSql = "insert into " + tfOverLoadTable + " values(" +
                        "'" + feeder + "','" + name + "','" + mRID + "','" + line[0] + "','" + line[1] + "','" + substation + "'," +
                        overLoadRate[0] + "," + overLoadRate[1] + "," + overLoadRate[2] + "," + overLoadRate[3] + ")";
                sqls.add(insertSql);
                sqliteDb.executeSqls(sqls);
                sqls.clear();
            }
        }
    }

    /**
     * 四季线路电流合格率
     */
    public void linePassRate(String dbFile, String feederTable, String substationTable, String swTable, String swToLineTable,
                             String oneLineParamTable, String linePassRateTable) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        // 查询馈线名称
        String feeder = sqliteDb.queryFeederName(feederTable);
        // 查询变电站名称
        String substation = sqliteDb.querySubstationName(substationTable);
        // 分析线路（开关对应的线路）
        Map<String, String> swNameToMRID = sqliteDb.queryNameToMRID(swTable);
        double[] passRate = new double[4];
        for (int season = 1; season < 5; season++) {
            int count = 0;
            for (String name : swNameToMRID.keySet()) {
                String mRID = swNameToMRID.get(name);
                List<String[]> lines = sqliteDb.querySwToDev(swToLineTable, mRID);  // 开关对应的线路
                List<SwitchData> swDatas = sqliteDb.querySwitchData(swTable, mRID, season);
                Iterator<SwitchData> iterator = swDatas.iterator();
                while (iterator.hasNext()) {
                    SwitchData swData = iterator.next();
                    double I = swData.getIa();
                    boolean pass = true;
                    for (String[] line : lines) {
                        double lineRatedI = sqliteDb.queryOneLineParam(oneLineParamTable, line[1]);
                        double loadRate = I / lineRatedI;
                        if (loadRate < 2 && loadRate > 1) {
                            pass = false;
                            break;
                        }
                    }
                    if (pass) {
                        passRate[season - 1]++;
                    }
                    count++;
                }
            }
            passRate[season - 1] /= count;
        }
        List<String> sqls = new LinkedList<>();
        String insertSql = "insert into " + linePassRateTable + " values(" +
                "'" + feeder + "','" + substation + "'," + passRate[0] + "," + passRate[1] + "," + passRate[2] + "," + passRate[3] + ")";
        sqls.add(insertSql);
        sqliteDb.executeSqls(sqls);
        sqls.clear();
    }

    /**
     * 计算三相相量不平衡度
     * @param Fa a相相量
     * @param Fb b相相量
     * @param Fc c相相量
     * @return 负序和零序不平衡度
     */
    public double[] unbalance(Complex Fa, Complex Fb, Complex Fc) {
        // 处理三相量为0的情况
        if (Fa.mod() + Fb.mod() + Fc.mod() < 1e-8)
            return new double[]{0, 0};
        double modFa1 = Complex.multiply(aThird, Complex.add(Complex.add(Fa, Complex.multiply(a, Fb)), Complex.multiply(a2, Fc))).mod();
        double modFa2 = Complex.multiply(aThird, Complex.add(Complex.add(Fa, Complex.multiply(a2, Fb)), Complex.multiply(a, Fc))).mod();
        double modFa0 = Complex.multiply(aThird, Complex.add(Complex.add(Fa, Fb), Fc)).mod();
        return new double[]{modFa2 / modFa1, modFa0 / modFa1};
    }
}
