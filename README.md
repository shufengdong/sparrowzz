# SGOOL网页主题

这是一个hexo主题文件，为了能够方便从md文件生成新闻公告和各种其他的网页，故使用[hexo](https://hexo.io/zh-cn/)来对md进行渲染，我将网站做成了hexo主题，后期的维护人员可以先学习如何使用hexo 

## 使用方法
1. 在自己的电脑上，安装hexo，新建一个，hexo项目，并下载这个github中的文件，放置到项目的theme文件下下面，将项目的 _config.yml中部署的设置修改成我们的服务地址，以方便部署

2. 下载source文件放到项目的source
3. 接下来就可以添加各种文件了
4. 添加新闻公告，可以在项目的source的_posts文件夹内添加，注意写好categoroies 和 tags
5. 添加人员，可以在主题的source中添加
6. 添加其他的文件可以联系开发人员

## 添加新的开发人员

在source/about文件里面添加相应的md文件即可，可以参考已有的示例，注意md最上面的元文件，要写清楚，其中pic是对应的照片路径，categories必须写member。 hexo generate会自动生成相应的页面，但是，还需要在about.ejs里面添加相应的页面链接

## 添加新的新闻公告

在source/_post里面添加新的md就可以了，注意元文件的填写，可以参考其他的示例

