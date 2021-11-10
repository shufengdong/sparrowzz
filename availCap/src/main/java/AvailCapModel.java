import org.jgrapht.UndirectedGraph;
import zju.devmodel.MapObject;

import java.util.*;

import static java.lang.Math.sqrt;

public class AvailCapModel {

    JyPowerSystem ps;
    Map<Edge, double[][]> edgeAvailCap = new HashMap<>();
    List<Edge> edges;
    Map<String, Edge> idToEdge = new HashMap<>();
    Map<String, Integer> edgeDist = new HashMap<>();
    Map<String, int[]> cnsPath = new HashMap<>();
    String[] loadPos = new String[4];
    int T = 15; // 采点时间间隔
    int pointNum = 24 * 60 / T;

    double JKsm = 1.09;
    double JKwin = 1.52;
    double LGJsm = 0.88;
    double LGJwin = 1.15;

    public AvailCapModel(JyPowerSystem ps) {
        this.ps = ps;
    }

    public AvailCapModel() {};

    /**
     * 创建变电站名称表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createSubStationNameTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " substation     varchar(200) NOT NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 存储变电站名称
     */
    public void saveSubstationName(String substationNameTable, String dbFile, String substationName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        List<String> sqls = new LinkedList<>();
        String insertSql = "insert into " + substationNameTable + " values(" +
                "'" + substationName + "')";
        sqls.add(insertSql);
        sqliteDb.executeSqls(sqls);
        sqls.clear();
    }

    /**
     * 创建线路限额表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createOneLineParamTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName     varchar(200) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " ratedCurrent       decimal(6,2) NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建可装容量分析结果表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createAvailCapTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName     varchar(50) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " season     INTEGER NULL," +
                " availCap        decimal(8,4) NULL " +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建负荷接入分析结果表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createLoadPosTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        if (!sqliteDb.existTable(tableName)) {
            String initSql = "CREATE TABLE " + tableName + " (" +
                    " loadId     INTEGER NOT NULL," +
                    " substation     varchar(20) NOT NULL," +
                    " feederName     varchar(20) NOT NULL," +
                    " season     INTEGER NULL," +
                    " canIn     varchar(8) NOT NULL," +
                    " tfName     varchar(50) NOT NULL," +
                    " tfMRID     varchar(50) NOT NULL," +
                    " phase     varchar(8) NOT NULL," +
                    " time     varchar(20) NOT NULL," +
                    " swName     varchar(50) NOT NULL," +
                    " swMRID     varchar(50) NOT NULL," +
                    " newLoadI      varchar(1000) NULL, " +
                    " swOrgLoad      varchar(1000) NULL, " +
                    " swNewLoad      varchar(1000) NULL, " +
                    " swRateI      decimal(6,2) NULL, " +
                    " newLoad      varchar(1000) NULL, " +
                    " tfOrgLoad      varchar(1000) NULL, " +
                    " tfNewLoad      varchar(1000) NULL, " +
                    " tfRateCap      decimal(6,2) NULL " +
                    ")";
            sqliteDb.initDb(initSql);
            // 清空表格
            String sql = "delete from " + tableName;
            sqliteDb.executeSql(sql);
        }
    }

    /**
     * 创建开关对应公变、公变对应开关的表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createSwToTfTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName1     varchar(50) NOT NULL," +
                " mRID1     varchar(50) NOT NULL," +
                " devName2     varchar(50) NOT NULL," +
                " mRID2     varchar(50) NOT NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建公变对应线路的表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createTfToLineTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName1     varchar(50) NOT NULL," +
                " mRID1     varchar(50) NOT NULL," +
                " devName2     varchar(50) NOT NULL," +
                " mRID2     varchar(50) NOT NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建开关对应线路、线路对应开关的表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createSwToLineTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " devName1     varchar(50) NOT NULL," +
                " mRID1     varchar(50) NOT NULL," +
                " devName2     varchar(50) NOT NULL," +
                " mRID2     varchar(50) NOT NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建线路预警表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createLineWarnTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " loadState     INTEGER NULL," +    // 1为重载，2为超载
                " devName     varchar(200) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " substation     varchar(20) NOT NULL," +
                " switchName     varchar(200) NOT NULL," +
                " switchMRID     varchar(50) NOT NULL," +
                " maxI       decimal(6,2) NULL," +
                " ratedCurrent       decimal(6,2) NULL," +
                " loadRate       decimal(4,5) NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建公变预警表格
     * @param dbFile 数据库路径
     * @param tableName 表名
     */
    public void createTfWarnTable(String dbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " loadState     INTEGER NULL," +    // 1为重载，2为超载
                " devName     varchar(200) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " lineName     varchar(200) NOT NULL," +
                " lineMRID     varchar(50) NOT NULL," +
                " substation     varchar(20) NOT NULL," +
                " maxLoad        decimal(8,4) NULL," +
                " ratedCap       decimal(4,0) NULL," +
                " loadRate       decimal(4,5) NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建所有馈线名称表格
     * @param allPsDbFile 数据库路径
     * @param tableName 表名
     */
    public void createAllFeederNameTable(String allPsDbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        if (!sqliteDb.existTable(tableName)) {
            String initSql = "CREATE TABLE " + tableName + " (" +
                    " feeder     varchar(200) NOT NULL" +
                    ")";
            sqliteDb.initDb(initSql);
            // 清空表格
            String sql = "delete from " + tableName;
            sqliteDb.executeSql(sql);
        }
    }

    /**
     * 存储所有馈线名称表格
     * @param allPsDbFile 数据库路径
     * @param tableName 表名
     */
    public void saveAllFeederNameTable(String allPsDbFile, String tableName, String feederName) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        List<String> sqls = new LinkedList<>();
        String insertSql = "insert into " + tableName + " values(" +
                "'" + feederName + "')";
        sqls.add(insertSql);
        sqliteDb.executeSqls(sqls);
        sqls.clear();
    }

    /**
     * 创建所有馈线预警表格
     * @param allPsDbFile 数据库路径
     * @param tableName 表名
     */
    public void createAllPsWarnTable(String allPsDbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " loadState     INTEGER NULL," +    // 1为重载，2为超载
                " type     INTEGER NULL," +    // 1为线路，2为配变
                " number     INTEGER NULL" +    // 数量
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建所有馈线线路预警表格
     * @param allPsDbFile 数据库路径
     * @param tableName 表名
     */
    public void createAllPsLineWarnTable(String allPsDbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " loadState     INTEGER NULL," +    // 1为重载，2为超载
                " devName     varchar(200) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " substation     varchar(20) NOT NULL," +
                " switchName     varchar(200) NOT NULL," +
                " switchMRID     varchar(50) NOT NULL," +
                " maxI       decimal(6,2) NULL," +
                " ratedCurrent       decimal(6,2) NULL," +
                " loadRate       decimal(4,5) NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建所有馈线公变预警表格
     * @param allPsDbFile 所有馈线数据库路径
     * @param tableName 表名
     */
    public void createAllPsTfWarnTable(String allPsDbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " loadState     INTEGER NULL," +    // 1为重载，2为超载
                " devName     varchar(200) NOT NULL," +
                " mRID     varchar(50) NOT NULL," +
                " lineName     varchar(200) NOT NULL," +
                " lineMRID     varchar(50) NOT NULL," +
                " substation     varchar(20) NOT NULL," +
                " maxLoad        decimal(8,4) NULL," +
                " ratedCap       decimal(4,0) NULL," +
                " loadRate       decimal(4,5) NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    /**
     * 创建线路可开放容量最大和最小两条表格
     * @param allPsDbFile 所有馈线数据库路径
     * @param tableName 表名
     */
    public void createMaxMinAvailCap(String allPsDbFile, String tableName) {
        SqliteDb sqliteDb = new SqliteDb(allPsDbFile);
        String initSql = "CREATE TABLE "  + tableName + " (" +
                " maxOrMin     INTEGER NULL," +    // 1为最大，2为最小
                " feederName     varchar(200) NOT NULL," +
                " availCap        decimal(8,4) NULL" +
                ")";
        sqliteDb.initDb(initSql);
        // 清空表格
        String sql = "delete from " + tableName;
        sqliteDb.executeSql(sql);
    }

    public void buildPaths() {
        if (ps.getSourceCns().isEmpty()) {
            System.out.println("No source find while build paths");
            return;
        }
        String supplyCn = ps.getSourceCns().get(0);
        UndirectedGraph<String, Edge> g = ps.getActiveIslands().get(0);
        edges = new ArrayList<>(g.edgeSet());
        for (Edge edge : edges) {
            idToEdge.put(edge.getEquips().get(0), edge);
        }
        // 初始化节点为未访问状态
        HashMap<String, Boolean> visited = new HashMap<>(g.vertexSet().size());
        for (String cn : g.vertexSet()) {
            visited.put(cn, false);
        }
        //用于深度优先搜索的栈
        Deque<String> stack = new ArrayDeque<>();
        Deque<Edge> edgeStack = new ArrayDeque<>();
        stack.push(supplyCn);   // 将电源节点压入栈内
        while (!stack.isEmpty()) {
            boolean noPush = true;
            String cn = stack.peek();
            for (Edge e: g.edgesOf(cn)) {
                if (!edgeStack.isEmpty() && edgeStack.peek().equals(e))
                    continue;
                edgeDist.put(e.getEquips().getFirst().substring(3), edgeStack.size());
                String neighbor = g.getEdgeTarget(e);
                if (neighbor.equals(cn))
                    neighbor = g.getEdgeSource(e);
                // 如果顶点已经被遍历过，则不进行处理
                if (visited.get(neighbor))
                    continue;
                // 未遍历过的节点
                stack.push(neighbor);
                noPush = false;
                edgeStack.push(e);
                int[] path = new int[edges.size()];
                for (Edge edge : edgeStack) {
                    for (int i = 0; i < edges.size(); i++) {
                        if (edges.get(i).equals(edge)) {
                            path[i] = 1;
                            break;
                        }
                    }
                }
                cnsPath.put(neighbor, path);
                break;
            }
            if (noPush) {
                visited.put(stack.pop(), true);
                if (!edgeStack.isEmpty()) {
                    edgeStack.pop();
                }
            }
        }
    }

    /**
     * 存储智能开关对应的公变，公变对应的智能开关，公变对应的线路
     */
    public void switchToTf(String switchTableName, String dbFile, String swToTfTableName, String tfToLineTableName) {
        // 获取开关的mRId
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        List<String> mRIDs = sqliteDb.queryMRIds(switchTableName);
        if (ps.getSourceCns().isEmpty()) {
            System.out.println("No source find while build paths");
            return;
        }
        String supplyCn = ps.getSourceCns().get(0);
        UndirectedGraph<String, Edge> g = ps.getActiveIslands().get(0);
        // 初始化节点为未访问状态
        HashMap<String, Boolean> visited = new HashMap<>(g.vertexSet().size());
        for (String cn : g.vertexSet()) {
            visited.put(cn, false);
        }
        //用于深度优先搜索的栈
        Deque<String> stack = new ArrayDeque<>();
        Deque<Edge> edgeStack = new ArrayDeque<>();
        Deque<String> switchStack = new ArrayDeque<>();   // 存储开关的栈
        Deque<String> lineStack = new ArrayDeque<>();   // 存储线路的栈
        stack.push(supplyCn);   // 将电源节点压入栈内
        while (!stack.isEmpty()) {
            boolean noPush = true;
            String cn = stack.peek();
            for (Edge e: g.edgesOf(cn)) {
                if (!edgeStack.isEmpty() && edgeStack.peek().equals(e))
                    continue;
                String neighbor = g.getEdgeTarget(e);
                if (neighbor.equals(cn))
                    neighbor = g.getEdgeSource(e);
                // 如果顶点已经被遍历过，则不进行处理
                if (visited.get(neighbor))
                    continue;
                // 未遍历过的节点
                stack.push(neighbor);
                noPush = false;
                edgeStack.push(e);
                // 智能开关数据库中查找
                String eId = e.getEquips().getFirst();
                if (ps.isSwitch(eId) && mRIDs.contains(eId.substring(3))) {
                        switchStack.push(eId);
                }
                // 线路入栈
                if (ps.getResource(eId).getType().equals("ACLINESEGMENT")) {
                    lineStack.push(eId);
                }
                break;
            }
            if (noPush) {
                visited.put(stack.pop(), true);
                if (!edgeStack.isEmpty()) {
                    Edge e = edgeStack.pop();
                    String eId = e.getEquips().getFirst();
                    if (ps.getResource(eId).getType().equals("POWERTRANSFORMER")) {
                        String tfName = ps.getResource(eId).getProperty("NAME");
                        // 存开关和公变对应数据
                        if (switchStack.size() > 0) {
                            String swId = switchStack.peek();
                            String swName = ps.getResource(swId).getProperty("NAME");
                            List<String> sqls = new LinkedList<>();
                            String insertSql = "insert into " + swToTfTableName + " values(" +
                                    "'" + swName + "','" + swId.substring(3) +
                                    "','" + tfName + "','" + eId.substring(3) + "')";
                            sqls.add(insertSql);
                            insertSql = "insert into " + swToTfTableName + " values(" +
                                    "'" + tfName + "','" + eId.substring(3) +
                                    "','" + swName + "','" + swId.substring(3) + "')";
                            sqls.add(insertSql);
                            sqliteDb.executeSqls(sqls);
                            sqls.clear();
                        }
                        // 存公变和线路对应数据
                        if (lineStack.size() > 0 ) {
                            String lineId = lineStack.peek();
                            String lineName = ps.getResource(lineId).getProperty("NAME");
                            List<String> sqls = new LinkedList<>();
                            String insertSql = "insert into " + tfToLineTableName + " values(" +
                                    "'" + tfName + "','" + eId.substring(3) +
                                    "','" + lineName + "','" + lineId.substring(3) + "')";
                            sqls.add(insertSql);
                            sqliteDb.executeSqls(sqls);
                            sqls.clear();
                        }
                    }
                    // 智能开关数据库中查找
                    if (ps.isSwitch(eId) && mRIDs.contains(eId.substring(3))) {
                        switchStack.pop();
                    }
                    // 线路
                    if (ps.getResource(eId).getType().equals("ACLINESEGMENT")) {
                        lineStack.pop();
                    }
                }
            }
        }
    }

    /**
     * 存储智能开关对应的线路，线路对应的智能开关
     */
    public void switchToLine(String switchTableName, String dbFile, String swToLineTableName) {
        // 获取开关的mRId
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        List<String> mRIDs = sqliteDb.queryMRIds(switchTableName);
        if (ps.getSourceCns().isEmpty()) {
            System.out.println("No source find while build paths");
            return;
        }
        String supplyCn = ps.getSourceCns().get(0);
        UndirectedGraph<String, Edge> g = ps.getActiveIslands().get(0);
        // 初始化节点为未访问状态
        HashMap<String, Boolean> visited = new HashMap<>(g.vertexSet().size());
        for (String cn : g.vertexSet()) {
            visited.put(cn, false);
        }
        //用于深度优先搜索的栈
        Deque<String> stack = new ArrayDeque<>();
        Deque<Edge> edgeStack = new ArrayDeque<>();
        String switchEdge = null;   // 线路段开关
        List<String> lines = new LinkedList<>();  // 线路段
        stack.push(supplyCn);   // 将电源节点压入栈内
        while (!stack.isEmpty()) {
            boolean noPush = true;
            String cn = stack.peek();
            for (Edge e: g.edgesOf(cn)) {
                if (!edgeStack.isEmpty() && edgeStack.peek().equals(e))
                    continue;
                String neighbor = g.getEdgeTarget(e);
                if (neighbor.equals(cn))
                    neighbor = g.getEdgeSource(e);
                // 如果顶点已经被遍历过，则不进行处理
                if (visited.get(neighbor))
                    continue;
                // 未遍历过的节点
                stack.push(neighbor);
                noPush = false;
                edgeStack.push(e);
                String eId = e.getEquips().getFirst();
                if (stack.size() > 2 && g.edgesOf(cn).size() > 2) {
                    // 存库
                    if (switchEdge != null) {
                        List<String> sqls = new LinkedList<>();
                        for (String line : lines) {
                            String swName = ps.getResource(switchEdge).getProperty("NAME");
                            String lineName = ps.getResource(line).getProperty("NAME");
                            String insertSql = "insert into " + swToLineTableName + " values(" +
                                    "'" + swName + "','" + switchEdge.substring(3) +
                                    "','" + lineName + "','" + line.substring(3) + "')";
                            sqls.add(insertSql);
                            insertSql = "insert into " + swToLineTableName + " values(" +
                                    "'" + lineName + "','" + line.substring(3) +
                                    "','" + swName + "','" + switchEdge.substring(3) + "')";
                            sqls.add(insertSql);
                        }
                        sqliteDb.executeSqls(sqls);
                        sqls.clear();
                    }
                    lines.clear();
                    switchEdge = null;
                }
                if (ps.getResource(eId).getType().equals("ACLINESEGMENT")) {
                    lines.add(eId);
                }
                // 智能开关数据库中查找
                if (ps.isSwitch(eId) && mRIDs.contains(eId.substring(3))) {
                    switchEdge = eId;
                }
                break;
            }
            if (noPush) {
                visited.put(stack.pop(), true);
                if (!edgeStack.isEmpty()) {
                    edgeStack.pop();
                }
            }
        }
    }

    public void calAvailCap(String resultTableName, String switchTableName, String dbFile) {
        if (ps.getSourceCns().isEmpty()) {
            System.out.println("No source find while calculate avail capacity");
            return;
        }
        String supplyCn = ps.getSourceCns().get(0);
        UndirectedGraph<String, Edge> g = ps.getActiveIslands().get(0);
        // 初始化节点为未访问状态
        HashMap<String, Boolean> visited = new HashMap<>(g.vertexSet().size());
        for (String cn : g.vertexSet()) {
            visited.put(cn, false);
        }
        //用于深度优先搜索的栈
        Deque<String> stack = new ArrayDeque<>();
        Deque<Edge> edgeStack = new ArrayDeque<>();
        stack.push(supplyCn);   // 将电源节点压入栈内
        while (!stack.isEmpty()) {
            boolean noPush = true;
            String cn = stack.peek();
            for (Edge e: g.edgesOf(cn)) {
                if (!edgeStack.isEmpty() && edgeStack.peek().equals(e))
                    continue;
                String neighbor = g.getEdgeTarget(e);
                if (neighbor.equals(cn))
                    neighbor = g.getEdgeSource(e);
                // 如果顶点已经被遍历过，则不进行处理
                if (visited.get(neighbor))
                    continue;
                // 未遍历过的节点
                stack.push(neighbor);
                noPush = false;
                double[][] availCap = edgeAvailCap.get(e);
                for (Edge edge : edgeStack) {
                    double[][] pathAvailCap = edgeAvailCap.get(edge);
                    for (int season = 0; season < 4; season++) {
                        for (int j = 0; j < pointNum; j++) {
                            if (availCap[season][j] > pathAvailCap[season][j]) {
                                availCap[season][j] = pathAvailCap[season][j];
//                                if (availCap[season][j] < 0) {
//                                    availCap[season][j] = 0;
//                                }
                            }
                        }
                    }
                }
                edgeAvailCap.put(e, availCap);
                edgeStack.push(e);
                break;
            }
            if (noPush) {
                visited.put(stack.pop(), true);
                if (!edgeStack.isEmpty()) {
                    edgeStack.pop();
                }
            }
        }
        // 存储可接入容量数据
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        List<String> mRIDs = sqliteDb.queryMRIds(switchTableName);
        for (String mRID : mRIDs) {
            Edge edge = idToEdge.get("PD_" + mRID);
            if (edge != null) {
                double[][] availCap = edgeAvailCap.get(edge);
                for (int season = 1; season < 5; season++) {
                    List<String> sqls = new LinkedList<>();
                    for (int j = 0; j < pointNum; j++) {
                        // 电流不转成容量
                        String insertSql = "insert into " + resultTableName + " values(" +
                                "'" + ps.getResource(edge.getEquips().get(0)).getProperty("NAME") + "','" + mRID +
                                "'," + season + "," + availCap[season - 1][j] + ")";
                        sqls.add(insertSql);
                    }
                    sqliteDb.executeSqls(sqls);
                    sqls.clear();
                }
            }
        }
    }

    public LoadPos loadPosOpt(double[] load, String swTableName, String availCapTableName, String dbFile,
                              String substationTable, String feederTable, String oneLineParamTable, String swClusterTable,
                              String swToTfTable, String tfParamTable, String tfAvailCapTable, String tfClusterTable,
                              String tfMinITable, String allPsdbFile, String loadPosTable) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        // 查询变电站名称
        String substation = sqliteDb.querySubstationName(substationTable);
        // 查询馈线名称
        String feeder = sqliteDb.queryFeederName(feederTable);
        double[] loadI = new double[pointNum];
        loadI[0] = load[0] / 10 / sqrt(3);
        StringBuilder newLoad = new StringBuilder(String.valueOf(load[0]));
        StringBuilder newLoadI = new StringBuilder(String.valueOf(loadI[0]));
        for (int i = 1; i < pointNum; i++) {
            loadI[i] = load[i] / 10 / sqrt(3);
            newLoad.append(";").append(load[i]);
            newLoadI.append(";").append(loadI[i]);
        }
        LoadPos loadPos = new LoadPos();
        loadPos.setNewLoad(load);
        loadPos.setNewLoadI(loadI);
        List<String> mRIDs = sqliteDb.queryMRIds(swTableName);
        // 最优开关
        String[] optEdge = new String[]{mRIDs.get(0), mRIDs.get(0), mRIDs.get(0), mRIDs.get(0)};
        double[] seasonAvgCap = new double[4]; // 负荷接入后裕度平均值
        double[] seasonMinCap = new double[4]; // 负荷接入后裕度最小值
        // 次优的开关
        String[] optEdge2 = new String[]{mRIDs.get(0), mRIDs.get(0), mRIDs.get(0), mRIDs.get(0)};
        double[] seasonAvgCap2 = new double[4];
        double[] seasonMinCap2 = new double[4];
        // 存储是否存在最优和次优的开关
        boolean[] hasFirst = new boolean[]{false, false, false, false};
        boolean[] hasSecond = new boolean[]{false, false, false, false};
        for (int season = 0; season < 4; season++) {
            for (String mRID : mRIDs) {
                double[] cap = sqliteDb.queryAvailCap(availCapTableName, mRID, season + 1, pointNum);
                double avgCap = 0;
                double minCap = Double.MAX_VALUE;
                boolean hasNag = false;
                for (int i = 0; i < pointNum; i++) {
                    double resCap = cap[i] - loadI[i];
                    if (resCap < 0) {
                        hasNag = true;
                        break;
                    }
                    avgCap += resCap;
                    if (minCap > resCap) {
                        minCap = resCap;
                    }
                }
                if (hasNag)
                    break;
                avgCap = avgCap / pointNum;
                if (avgCap + minCap > seasonAvgCap[season] + seasonMinCap[season]) {
                    // 存储次优数据
                    optEdge2[season] = optEdge[season];
                    seasonAvgCap2[season] = seasonAvgCap[season];
                    seasonMinCap2[season] = seasonMinCap[season];
                    if (hasFirst[season])
                        hasSecond[season] = true;
                    // 存储最优数据
                    optEdge[season] = mRID;
                    seasonAvgCap[season] = avgCap;
                    seasonMinCap[season] = minCap;
                    hasFirst[season] = true;
                } else if (avgCap + minCap == seasonAvgCap[season] + seasonMinCap[season]) {
                    // 优先接在离主线近的位置
                    // 存储次优数据
                    seasonAvgCap2[season] = seasonAvgCap[season];
                    seasonMinCap2[season] = seasonMinCap[season];
                    if (hasFirst[season])
                        hasSecond[season] = true;
                    // 存储最优数据
                    if (hasFirst[season]) {
                        if (edgeDist.get(mRID) < edgeDist.get(optEdge[season])) {
                            optEdge2[season] = optEdge[season];
                            optEdge[season] = mRID;
                        } else {
                            if (hasSecond[season]) {
                                if (edgeDist.get(mRID) < edgeDist.get(optEdge2[season])) {
                                    optEdge2[season] = mRID;
                                }
                            } else {
                                optEdge2[season] = mRID;
                            }
                        }
                    }
                }
            }
            // 时间
            long t = System.currentTimeMillis();
            Calendar calendar = Calendar.getInstance();
            calendar.setTimeInMillis(t);
            int year = calendar.get(Calendar.YEAR);
            int month = calendar.get(Calendar.MONTH) + 1;
            int day = calendar.get(Calendar.DAY_OF_MONTH);
            int hour = calendar.get(Calendar.HOUR_OF_DAY);
            int minutes = calendar.get(Calendar.MINUTE);
            String time = year + "." + month + "." + day + " " + hour + ":" + minutes;
            SqliteDb allPsDb = new SqliteDb(allPsdbFile);
            int loadId = allPsDb.queryMaxLoadId(loadPosTable) + 1;
            if (hasFirst[season]) {
                MapObject edge = ps.getResource("PD_" + optEdge[season]);
                if (edge != null) {
                    loadPos.addLoadPos(season, optEdge[season], edge.getProperty("NAME"), seasonAvgCap[season], seasonMinCap[season]);
                } else {
                    loadPos.addLoadPos(season, optEdge[season], "", seasonAvgCap[season], seasonMinCap[season]);
                }
//                System.out.println(season + " " + optEdge[season] + " " + seasonAvgCap[season] + " " + seasonMinCap[season]);
                // 查询开关的限额
                double swRatedI = sqliteDb.queryOneLineParam(oneLineParamTable, optEdge[season]);
                loadPos.getSwRateI()[season] = swRatedI;
                // 查询开关聚类电流
                double[] swCluster = sqliteDb.querySeasonSwitchI(swClusterTable, optEdge[season], season + 1, pointNum);
                loadPos.setSwOrgLoad(season, swCluster);
                loadPos.calSwNewLoad(season);
                // 查询开关对应的公变
                List<String[]> tfs = sqliteDb.querySwToDev(swToTfTable, optEdge[season]);
                boolean tfCanIn = false;
                double optTfRateCap = 400;
                String optTf = null;
                String optTfName = null;
                double maxSumAvailCap = 0;
                for (String[] tf : tfs) {
                    // 查询公变可开放容量
                    double[] tfAvailCap = sqliteDb.queryAvailCap(tfAvailCapTable, tf[1], season + 1, pointNum);
                    boolean canIn = true;
                    double sumAvailCap = 0;
                    for (int i = 0; i < pointNum; i++) {
                        sumAvailCap += tfAvailCap[i];
                        if (tfAvailCap[i] < load[i]) {
                            canIn = false;
                            break;
                        }
                    }
                    if (!canIn)
                        continue;
                    tfCanIn = true;
                    optTfRateCap = sqliteDb.queryTFCap(tfParamTable, tf[1]);
                    if (sumAvailCap > maxSumAvailCap) {
                        maxSumAvailCap = sumAvailCap;
                        optTf = tf[1];
                        optTfName = tf[0];
                    }
                }
                loadPos.getCanIn()[season] = tfCanIn;
                if (tfCanIn) {
                    loadPos.getTfRateCap()[season] = optTfRateCap;
                    loadPos.getOptTfs()[season] = optTf;
                    loadPos.getOptTfNames()[season] = optTfName;
                    // 查询公变聚类负荷
                    double[] tfCluster = sqliteDb.querySeasonTFP(tfClusterTable, optTf, season + 1, pointNum);
                    for (int i = 0; i < pointNum; i++) {
                        tfCluster[i] /= 1000;
                    }
                    loadPos.setTfOrgLoad(season, tfCluster);
                    loadPos.calTfNewLoad(season);
                    // 查询公变接入相别
                    String phase = sqliteDb.queryMinIPhase(tfMinITable, optTf, season + 1);
                    loadPos.getOptPhase()[season] = phase;
                    StringBuilder swOrgLoad = new StringBuilder(String.valueOf(loadPos.getSwOrgLoad()[season][0]));
                    StringBuilder swNewLoad = new StringBuilder(String.valueOf(loadPos.getSwNewLoad()[season][0]));
                    StringBuilder tfOrgLoad = new StringBuilder(String.valueOf(loadPos.getTfOrgLoad()[season][0]));
                    StringBuilder tfNewLoad = new StringBuilder(String.valueOf(loadPos.getTfNewLoad()[season][0]));
                    for (int i = 1; i < pointNum; i++) {
                        swOrgLoad.append(";").append(loadPos.getSwOrgLoad()[season][i]);
                        swNewLoad.append(";").append(loadPos.getSwNewLoad()[season][i]);
                        tfOrgLoad.append(";").append(loadPos.getTfOrgLoad()[season][i]);
                        tfNewLoad.append(";").append(loadPos.getTfNewLoad()[season][i]);
                    }
                    List<String> sqls = new LinkedList<>();
                    // 开关电流，公变容量kW
                    String insertSql = "insert into " + loadPosTable + " values(" +
                            loadId + ",'" + substation + "','" + feeder + "'," + (season + 1) + ",'是','" + optTfName +
                            "','" + optTf + "','" + phase + "相','" + time + "','" + optEdge[season] + "','" + edge.getProperty("NAME") +
                            "','" + newLoadI + "','" + swOrgLoad + "','" + swNewLoad + "'," + loadPos.getSwRateI()[season] +
                            ",'" + newLoad + "','" + tfOrgLoad + "','" + tfNewLoad + "'," + optTfRateCap + ")";
                    sqls.add(insertSql);
                    allPsDb.executeSqls(sqls);
                    sqls.clear();
                } else {
                    List<String> sqls = new LinkedList<>();
                    String insertSql = "insert into " + loadPosTable + " values(" +
                            loadId + ",'" + substation + "','" + feeder + "'," + (season + 1) + ",'否','" + "" +
                            "','" + "" + "','" + "" + "','" + time + "','" + "" + "','" + "" +
                            "','" + newLoad + "','" + "" + "','" + "" + "'," + 0 +
                            ",'" + newLoadI + "','" + "" + "','" + "" + "'," + 0 + ")";
                    sqls.add(insertSql);
                    allPsDb.executeSqls(sqls);
                    sqls.clear();
                }
            } else {
                loadPos.getCanIn()[season] = false;
                List<String> sqls = new LinkedList<>();
                String insertSql = "insert into " + loadPosTable + " values(" +
                        loadId + ",'" + substation + "','" + feeder + "'," + (season + 1) + ",'否','" + " " +
                        "','" + "" + "','" + "" + "','" + time + "','" + "" + "','" + "" +
                        "','" + newLoad + "','" + "" + "','" + "" + "'," + 0 +
                        ",'" + newLoadI + "','" + "" + "','" + "" + "'," + 0 + ")";
                sqls.add(insertSql);
                allPsDb.executeSqls(sqls);
                sqls.clear();
            }
            if (hasSecond[season]) {
                MapObject edge = ps.getResource("PD_" + optEdge2[season]);
                if (edge != null) {
                    loadPos.addLoadPos2(season, optEdge2[season], edge.getProperty("NAME"), seasonAvgCap2[season], seasonMinCap2[season]);
                } else {
                    loadPos.addLoadPos2(season, optEdge2[season], "", seasonAvgCap2[season], seasonMinCap2[season]);
                }
//                System.out.println(season + " " + optEdge2[season] + " " + seasonAvgCap2[season] + " " + seasonMinCap2[season]);
            }
        }
        return loadPos;
    }

    /**
     * 设置支路电流限值
     * @param tableName 表名
     * @param dbFile 数据库名
     */
    public void setEdgeLimI(String tableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        // 读取两个杆的名称
        List<LineParamData> lineParamDatas = sqliteDb.queryLineParam(tableName);
        for (LineParamData lineParamData : lineParamDatas) {
            String cn1 = ps.getPoleNameToCn().get(lineParamData.getPole1());
            String cn2 = ps.getPoleNameToCn().get(lineParamData.getPole2());
            if (cn1 != null && cn2 != null) {
                double limI = lineParamData.getRatedI();
                int[] path1 = cnsPath.get(cn1);
                int[] path2 = cnsPath.get(cn2);
                for (int i = 0; i < edges.size(); i++) {
                    if (path1[i] + path2[i] == 1) {
                        Edge e = edges.get(i);
                        e.setType(lineParamData.getType());
                        if (e.getLimI() > limI) {
                            e.setLimI(limI);
                        }
                    }
                }
            }
        }
    }

    /**
     * 设置支路电流限值并存库
     * @param tableName 表名
     * @param dbFile 数据库名
     * @param oneLineTableName 单线参数表名
     */
    public void setEdgeLimI(String tableName, String dbFile, String oneLineTableName) {
        setEdgeLimI(tableName, dbFile);
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        List<String> sqls = new LinkedList<>();
        for (Edge e : edges) {
            String insertSql = "insert into " + oneLineTableName + " values(" +
                    "'" + ps.getResource(e.getEquips().get(0)).getProperty("NAME") +
                    "','" + e.getEquips().get(0).substring(3) + "'," + e.getLimI() + ")";
            sqls.add(insertSql);
        }
        sqliteDb.executeSqls(sqls);
        sqls.clear();
    }

    /**
     * 根据支路电流历史季度电流最大值，设置单支路电流裕度
     * @param lineISeasontableName 线路电流按季度分析结果表名
     * @param swISeasontableName 开关电流按季度分析结果表名
     * @param dbFile 数据库名
     */
    public void setEdgeAvailCap(String lineISeasontableName, String swISeasontableName, String dbFile) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        // 查询主线电流数据
        double[][] mainLineI = new double[4][pointNum];
        for (int season = 1; season < 5; season++) {
            int pNTimes = 3;
            double[] I = sqliteDb.querySeasonLineI(lineISeasontableName, season, pNTimes * pointNum);
            for (int i = 0; i < pointNum; i++) {
                mainLineI[season - 1][i] = I[pNTimes * i];
                if (mainLineI[season - 1][i] < I[pNTimes * i + 1]) {
                    mainLineI[season - 1][i] = I[pNTimes * i + 1];
                }
                if (mainLineI[season - 1][i] < I[pNTimes * i + 2]) {
                    mainLineI[season - 1][i] = I[pNTimes * i + 2];
                }
            }
        }
        // 主线电流裕度，按第一条有额定电流的线路算
        String supplyCn = ps.getSourceCns().get(0);
        UndirectedGraph<String, Edge> g = ps.getActiveIslands().get(0);
        Edge mainLine = g.edgesOf(supplyCn).iterator().next();
        // 初始化节点为未访问状态
        HashMap<String, Boolean> visited = new HashMap<>(g.vertexSet().size());
        for (String cn : g.vertexSet()) {
            visited.put(cn, false);
        }
        //用于深度优先搜索的栈
        Deque<String> stack = new ArrayDeque<>();
        Deque<Edge> edgeStack = new ArrayDeque<>();
        stack.push(supplyCn);   // 将电源节点压入栈内
        while (!stack.isEmpty()) {
            boolean noPush = true;
            String cn = stack.peek();
            for (Edge e: g.edgesOf(cn)) {
                if (!edgeStack.isEmpty() && edgeStack.peek().equals(e))
                    continue;
                if (e.getLimI() < 10000) {
                    mainLine = e;
                    stack.clear();
                    break;
                }
                String neighbor = g.getEdgeTarget(e);
                if (neighbor.equals(cn))
                    neighbor = g.getEdgeSource(e);
                // 如果顶点已经被遍历过，则不进行处理
                if (visited.get(neighbor))
                    continue;
                // 未遍历过的节点
                stack.push(neighbor);
                noPush = false;
                edgeStack.push(e);
                break;
            }
            if (noPush) {
                if (!stack.isEmpty()) {
                    visited.put(stack.pop(), true);
                }
                if (!edgeStack.isEmpty()) {
                    edgeStack.pop();
                }
            }
        }
        double mainLineLimI = mainLine.getLimI();
        double mainLineType = mainLine.getType();
        double[][] mainLineAvailCap = new double[4][pointNum];
        for (int season = 0; season < 4; season++) {
            for (int j = 0; j < pointNum; j++) {
                if (season == 1 || season == 2) {
                    if (mainLineType == 1) {
                        mainLineAvailCap[season][j] = mainLineLimI * JKsm - mainLineI[season][j];
                    } else if (mainLineType == 2) {
                        mainLineAvailCap[season][j] = mainLineLimI * LGJsm - mainLineI[season][j];
                    }
                } else {
                    if (mainLineType == 1) {
                        mainLineAvailCap[season][j] = mainLineLimI * JKwin - mainLineI[season][j];
                    } else if (mainLineType == 2) {
                        mainLineAvailCap[season][j] = mainLineLimI * LGJwin - mainLineI[season][j];
                    }
                }
            }
        }
//        for (UndirectedGraph<String, Edge> g : ps.getActiveIslands()) {
        for (Edge e : g.edgeSet()) {
            double[][] availCap = new double[4][pointNum];
            for (int i = 0; i < 4; i++) {
                for (int j = 0; j < pointNum; j++) {
                    availCap[i][j] = mainLineAvailCap[i][j];
                }
            }
            for (int season = 1; season < 5; season++) {
                for (String equip : e.getEquips()) {
                    String mRID = equip.substring(3);
                    double[] I = sqliteDb.querySeasonSwitchI(swISeasontableName, mRID, season, pointNum);
                    double limI = e.getLimI();
                    int type = e.getType();
                    for (int i = 0; i < pointNum; i++) {
                        if (season == 2 || season == 3) {
                            if (type == 1) {
                                if (availCap[season - 1][i] > limI * JKsm - I[i]) {
                                    availCap[season - 1][i] = limI * JKsm - I[i];
                                }
                            } else if (type == 2) {
                                if (availCap[season - 1][i] > limI * LGJsm - I[i]) {
                                    availCap[season - 1][i] = limI * LGJsm - I[i];
                                }
                            }
                        } else {
                            if (type == 1) {
                                if (availCap[season - 1][i] > limI * JKwin - I[i]) {
                                    availCap[season - 1][i] = limI * JKwin - I[i];
                                }
                            } else if (type == 2) {
                                if (availCap[season - 1][i] > limI * LGJwin - I[i]) {
                                    availCap[season - 1][i] = limI * LGJwin - I[i];
                                }
                            }
                        }
                    }
                }
            }
            edgeAvailCap.put(e, availCap);
        }
//        }
    }

    /**
     * 设备预警分析
     */
    public void warnDevAnalysis(String dbFile, String substationTable, String tfWarnTable, String tfTable, String tfParamTable, String tfSeasonTable,
                                String tfToLineTable, String swTable, String swToLineTable, String oneLineParamTable,
                                String swSeasonTable, String lineWarnTable) {
        SqliteDb sqliteDb = new SqliteDb(dbFile);
        // 查询变电站名称
        String substation = sqliteDb.querySubstationName(substationTable);
        // 分析公变
        Map<String, String> tfNameToMRID = sqliteDb.queryNameToMRID(tfTable);
        List<String> sqls = new LinkedList<>();
        for (String name : tfNameToMRID.keySet()) {
            String mRID = tfNameToMRID.get(name);
            double tfRatedCap = sqliteDb.queryTFCap(tfParamTable, mRID);
            double tfMaxP = sqliteDb.queryMaxTFP(tfSeasonTable, mRID, -1) / 1000;
            double loadRate = tfMaxP / tfRatedCap;
            // 公变对应的线路名称
            List<String[]> lines = sqliteDb.querySwToDev(tfToLineTable, mRID);
            if (lines.size() > 0) {
                String[] line = lines.get(0);
                if (loadRate < 2 && loadRate > 1) {
                    String insertSql = "insert into " + tfWarnTable + " values(" +
                            "2,'" + name + "','" + mRID + "','" + line[0] + "','" + line[1] + "','" + substation + "'," +
                            tfMaxP + "," + tfRatedCap + "," + loadRate + ")";
                    sqls.add(insertSql);
                } else if (loadRate < 1 && loadRate > 0.8) {
                    String insertSql = "insert into " + tfWarnTable + " values(" +
                            "1,'" + name + "','" + mRID + "','" + line[0] + "','" + line[1] + "','" + substation + "'," +
                            tfMaxP + "," + tfRatedCap + "," + loadRate + ")";
                    sqls.add(insertSql);
                }
            }
        }
        sqliteDb.executeSqls(sqls);
        sqls.clear();
        // 分析线路（开关对应的线路）
        Map<String, String> swNameToMRID = sqliteDb.queryNameToMRID(swTable);
        for (String name : swNameToMRID.keySet()) {
            String mRID = swNameToMRID.get(name);
            List<String[]> lines = sqliteDb.querySwToDev(swToLineTable, mRID);
            for (String[] line : lines) {
                double lineRatedI = sqliteDb.queryOneLineParam(oneLineParamTable, line[1]);
                double lineMaxI = sqliteDb.queryMaxSwitchI(swSeasonTable, mRID, -1);
                double loadRate = lineMaxI / lineRatedI;
                if (loadRate < 2 && loadRate > 1) {
                    String insertSql = "insert into " + lineWarnTable + " values(" +
                            "2,'" + line[0] + "','" + line[1] + "','" + substation + "','" + name + "','" + mRID +
                            "'," + lineMaxI + "," + lineRatedI + "," + loadRate + ")";
                    sqls.add(insertSql);
                } else if (loadRate < 1 && loadRate > 0.8) {
                    String insertSql = "insert into " + lineWarnTable + " values(" +
                            "1,'" + line[0] + "','" + line[1] + "','" + substation + "','" + name + "','" + mRID +
                            "'," + lineMaxI + "," + lineRatedI + "," + loadRate + ")";
                    sqls.add(insertSql);
                }
            }
        }
        sqliteDb.executeSqls(sqls);
        sqls.clear();
    }

    /**
     * 所有馈线预警设备数量统计，详细信息存库
     */
    public void allPsWarnDev(String allPsdbFile, String allFeederNameTable, String dbFilePath, String allPsWarnTable,
                             String allPsLineWarnTable, String allPsTfWarnTable) {
        SqliteDb allPsDb = new SqliteDb(allPsdbFile);
        // 查询馈线名称
        List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);
        int[] warnLineNum = new int[2];
        int[] warnTfNum = new int[2];
        // 处理馈线数据库文件
        for (String feeder : feeders) {
            SqliteDb sqliteDb = new SqliteDb(dbFilePath + "\\" + feeder + ".db");
            List<WarnLine> warnLines = sqliteDb.queryWarnLine(feeder + AvailCapMain.lineWarnTableName);
            List<String> sqls = new LinkedList<>();
            for (WarnLine warnLine : warnLines) {
                if (warnLine.loadState == 1) {
                    warnLineNum[0]++;
                } else {
                    warnLineNum[1]++;
                }
                String insertSql = "insert into " + allPsLineWarnTable + " values(" +
                        warnLine.loadState + ",'" + warnLine.devName + "','" + warnLine.mRID + "','" + warnLine.substation +
                        "','" + warnLine.switchName + "','" + warnLine.switchMRID + "'," + warnLine.maxI +
                        "," + warnLine.ratedCurrent + "," + warnLine.loadRate + ")";
                sqls.add(insertSql);
            }
            allPsDb.executeSqls(sqls);
            sqls.clear();
            List<WarnTf> warnTfs = sqliteDb.queryWarnTf(feeder + AvailCapMain.tfWarnTableName);
            for (WarnTf warnTf : warnTfs) {
                if (warnTf.loadState == 1) {
                    warnTfNum[0]++;
                } else {
                    warnTfNum[1]++;
                }
                String insertSql = "insert into " + allPsTfWarnTable + " values(" +
                        warnTf.loadState + ",'" + warnTf.devName + "','" + warnTf.mRID + "','" + warnTf.lineName + "','" +
                        warnTf.lineMRID + "','" + warnTf.substation + "'," + warnTf.maxLoad + "," + warnTf.ratedCap + "," + warnTf.loadRate + ")";
                sqls.add(insertSql);
            }
            allPsDb.executeSqls(sqls);
            sqls.clear();
        }
        List<String> sqls = new LinkedList<>();
        String insertSql = "insert into " + allPsWarnTable + " values(" +
                1 + "," + 1 + "," + warnLineNum[0] + ")";
        sqls.add(insertSql);
        insertSql = "insert into " + allPsWarnTable + " values(" +
                2 + "," + 1 + "," + warnLineNum[1] + ")";
        sqls.add(insertSql);
        insertSql = "insert into " + allPsWarnTable + " values(" +
                1 + "," + 2 + "," + warnTfNum[0] + ")";
        sqls.add(insertSql);
        insertSql = "insert into " + allPsWarnTable + " values(" +
                2 + "," + 2 + "," + warnTfNum[1] + ")";
        sqls.add(insertSql);
        allPsDb.executeSqls(sqls);
        sqls.clear();
    }

    /**
     * 线路可开放容量最大和最小两条
     */
    public void maxMinAvailCap(String allPsdbFile, String allFeederNameTable, String dbFilePath, String maxMinAvailCapTable) {
        SqliteDb allPsDb = new SqliteDb(allPsdbFile);
        // 查询馈线名称
        List<String> feeders = allPsDb.queryAllFeederName(allFeederNameTable);
        String maxFeederName = null;
        double maxFeederAvailCap = 0;
        String minFeederName = null;
        double minFeederAvailCap = 1e8;
        // 处理馈线数据库文件
        for (String feeder : feeders) {
            SqliteDb sqliteDb = new SqliteDb(dbFilePath + "\\" + feeder + ".db");
            // 查询所有开关的mRID
            List<String> mRIDs = sqliteDb.queryMRIds(feeder + AvailCapMain.switchTableName);
            double maxLineAvailCap = 0;
            for (String mRID : mRIDs) {
                double avgAvailCap = 0;
                int count = 0;
                for (int season = 1; season < 5; season++) {
                    double[] availCap = sqliteDb.queryAvailCap(feeder + AvailCapMain.availCapTableName, mRID, season, pointNum);
                    for (int i = 0; i < pointNum; i++) {
                        avgAvailCap += availCap[i];
                        count++;
                    }
                }
                avgAvailCap /= count;
                if (avgAvailCap > maxLineAvailCap) {
                    maxLineAvailCap = avgAvailCap;
                }
            }
            if (maxLineAvailCap > maxFeederAvailCap) {
                maxFeederAvailCap = maxLineAvailCap;
                maxFeederName = feeder;
            }
            if (maxLineAvailCap < minFeederAvailCap) {
                minFeederAvailCap = maxLineAvailCap;
                minFeederName = feeder;
            }
        }
        List<String> sqls = new LinkedList<>();
        String insertSql = "insert into " + maxMinAvailCapTable + " values(" +
                "1,'" + maxFeederName + "'," + maxFeederAvailCap + ")";
        sqls.add(insertSql);
        insertSql = "insert into " + maxMinAvailCapTable + " values(" +
                "2,'" + minFeederName + "'," + minFeederAvailCap + ")";
        sqls.add(insertSql);
        allPsDb.executeSqls(sqls);
        sqls.clear();
    }
}
